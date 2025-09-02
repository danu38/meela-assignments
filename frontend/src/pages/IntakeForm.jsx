import React, { useEffect, useMemo, useRef, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { getDraft, patchDraft, submitDraft } from "../api";

const box = {
  maxWidth: 720,
  margin: "40px auto",
  padding: 20,
  border: "1px solid #ddd",
  borderRadius: 12,
  fontFamily: "system-ui, sans-serif",
};
const row = {
  display: "flex",
  gap: 8,
  justifyContent: "space-between",
  alignItems: "center",
};

const steps = [
  { key: "basics", title: "Basics" },
  { key: "concern", title: "Main concern" },
  { key: "goals", title: "Goals" },
  { key: "background", title: "Background" },
];

const emptyForm = {
  fullName: "",
  email: "",
  mainConcern: "",
  goals: "",
  background: "",
};

export default function IntakeForm() {
  const { id } = useParams();
  const nav = useNavigate();

  const [form, setForm] = useState(emptyForm);
  const [step, setStep] = useState(0);
  const [status, setStatus] = useState("draft");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const dirtyRef = useRef(false);

  /* dirtyRef.current = true : the form has changes that haven’t been saved yet.

dirtyRef.current = false :all changes are persisted to the backend. */

  const timer = useRef(null);
  const emailIsValid = (value) => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);

  // Load existing draft
  useEffect(() => {
    (async () => {
      try {
        const d = await getDraft(id);
        setForm((prev) => ({ ...prev, ...d.data }));
        setStep(Math.min(steps.length - 1, d.step || 0)); //updates which step the user is on
        setStatus(d.status);
      } catch {
        alert("Draft not found");
        nav("/");
      } finally {
        setLoading(false);
      }
    })();
  }, [id, nav]);

  // Debounced autosave :delay execution of a function until a 600ms time has passed since the last call

  /*   Every time the user types, you call queueSave(nextForm).

It cancels the old timer, starts a new one.

If no typing happens for 600ms, the draft is saved to the backend.

useMemo makes sure this logic is stable between renders. */
  const queueSave = useMemo(
    () => (next) => {
      dirtyRef.current = true;
      if (timer.current) clearTimeout(timer.current); // reset old timer
      timer.current = setTimeout(async () => {
        setSaving(true);
        try {
          await patchDraft(id, { data: next, step }); // save to backend
        } finally {
          setSaving(false); // saved
          dirtyRef.current = false;
        }
      }, 600); //wait 600ms after the last keystroke, then save
    },
    [id, step]
  );

  const onChange = (patch) => {
    const next = { ...form, ...patch };
    setForm(next);
    queueSave(next);
  };

  const moveSteps = async (direction) => {
    const nextStep = Math.max(0, Math.min(steps.length - 1, step + direction));
    setStep(nextStep);
    setSaving(true);
    try {
      await patchDraft(id, { data: form, step: nextStep });
    } finally {
      setSaving(false);
    }
  };

  const saveAndExit = async () => {
    if (dirtyRef.current) {
      setSaving(true);
      try {
        await patchDraft(id, { data: form, step });
      } finally {
        setSaving(false);
        dirtyRef.current = false;
      }
    }
    const url = `${window.location.origin}/form/${id}`;
    try {
      await navigator.clipboard.writeText(url);
    } catch {}
    alert(`Resume link copied:\n${url}`);
    nav("/");
  };

  const canProceed = () => {
    if (step === 0) {
      const nameOk = form.fullName.trim().length > 0;
      const emailOk = emailIsValid(form.email);
      return nameOk && emailOk;
    }
    if (step === 1) return form.mainConcern.trim();
    if (step === 2) return form.goals.trim();
    if (step === 3) return form.background?.trim();
    return true;
  };
  const handleEmailChange = (e) => {
    const value = e.target.value;
    onChange({ email: value });

    if (!emailIsValid(value)) {
      setError("Please enter a valid email");
    } else {
      setError("");
    }
  };

  const onSubmit = async (e) => {
    e.preventDefault();
    setSaving(true);
    try {
      await patchDraft(id, { data: form, step });
      const res = await submitDraft(id);
      setStatus(res.status);
    } finally {
      setSaving(false);
    }
  };

  if (loading)
    return (
      <div style={box}>
        <p>Loading…</p>
      </div>
    );
  if (status === "submitted") {
    return (
      <div className="container">
        <h3 className="h1">Thanks!</h3>
        <p>Your intake was submitted.</p>
        <Link to="/" className="btn btn-outline">
          Back to start
        </Link>
      </div>
    );
  }

  return (
    <div className="container">
      <div className="toprow">
        <div>
          <div className="h2">
            Step {step + 1} — {steps[step].title}
          </div>
          <div className="stepper">
            {steps.map((_, i) => (
              <div key={i} className={`step ${i <= step ? "active" : ""}`} />
            ))}
          </div>
        </div>
        <div className="badge">
          {saving ? "Saving…" : "Saved"}{" "}
          <button
            type="button"
            className="btn btn-outline"
            onClick={saveAndExit}
          >
            Save & Exit
          </button>
        </div>
      </div>

      <form
        onSubmit={onSubmit}
        onKeyDown={(e) => {
          if (e.key === "Enter" && e.target.tagName === "INPUT")
            e.preventDefault();
        }}
      >
        {step === 0 && (
          <>
            <label>Full name</label>
            <input
              className="input"
              value={form.fullName}
              onChange={(e) => onChange({ fullName: e.target.value })}
              placeholder="Danu Smith"
            />

            <label>Email</label>
            <input
              className="input"
              type="email"
              value={form.email}
              onChange={handleEmailChange}
              placeholder="danu@gmail.com"
              required
            />
            {error && <p style={{ color: "red" }}>{error}</p>}
          </>
        )}

        {step === 1 && (
          <>
            <label>What brings you to therapy right now?</label>
            <textarea
              className="textarea"
              rows="5"
              value={form.mainConcern}
              onChange={(e) => onChange({ mainConcern: e.target.value })}
            />
          </>
        )}

        {step === 2 && (
          <>
            <label>What would you like to get out of therapy?</label>
            <textarea
              className="textarea"
              rows="5"
              value={form.goals}
              onChange={(e) => onChange({ goals: e.target.value })}
            />
          </>
        )}

        {step === 3 && (
          <>
            <label>Please tell us a little about your background</label>
            <textarea
              className="textarea"
              rows="5"
              value={form.background}
              onChange={(e) => onChange({ background: e.target.value })}
            />
          </>
        )}
        <div
          style={{
            marginTop: 16,
            display: "flex",
            justifyContent: "space-between",
          }}
        >
          <div className="actions">
            {/* Back button */}
            <button
              type="button"
              className="btn btn-outline"
              onClick={() => moveSteps(-1)}
              disabled={step === 0}
            >
              Back
            </button>

            {step < steps.length - 1 ? (
              <button
                type="button"
                className="btn btn-primary"
                onClick={() => moveSteps(+1)}
                disabled={!canProceed()}
                title={!canProceed() ? "Please fill required fields" : ""}
              >
                Next
              </button>
            ) : (
              <button
                type="submit"
                className="btn btn-primary"
                disabled={!canProceed()}
                title={!canProceed() ? "Please fill required fields" : ""}
              >
                Submit
              </button>
            )}
          </div>
        </div>
      </form>

      <p style={{ opacity: 0.7, marginTop: 12 }}>
        Resume any time: <code>/form/{id}</code>
      </p>
    </div>
  );
}
