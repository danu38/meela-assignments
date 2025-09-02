// Uses Vite proxy, so calls go to http://localhost:3005
export async function createDraft() {
  const r = await fetch("/api/drafts", { method: "POST" });
  if (!r.ok) throw new Error("create failed");
  return r.json();
}

export async function getDraft(id) {
  const r = await fetch(`/api/drafts/${id}`);
  if (!r.ok) throw new Error("not found");
  return r.json();
}

export async function patchDraft(id, body) {
  const r = await fetch(`/api/drafts/${id}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!r.ok) throw new Error("save failed");
  return r.json();
}

export async function submitDraft(id) {
  const r = await fetch(`/api/drafts/${id}/submit`, { method: "POST" });
  if (!r.ok) throw new Error("submit failed");
  return r.json();
}
