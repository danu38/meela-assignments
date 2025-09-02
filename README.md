# Meela Intake – Frontend (React + Vite)

A tiny multi-step **client intake form** with **autosave** and **resume via UUID**.  
Talks to the backend at `/api/*` (proxied to `http://localhost:3005` in dev).

---

## Quick start

Prereqs:
- Node.js ≥ 18
- Backend running locally on **port 3005** (`cargo run` in `backend/`)

Run:
```bash
cd frontend
npm install
npm run dev
# open http://localhost:5173


# Meela Intake – Backend (Poem + MongoDB)

Small REST API that supports **partial save + resume** for a multi-step intake form.

- **Framework:** [Poem]
- **Language:** Rust (Tokio async)
- **DB:** MongoDB Atlas
- **Style:** No auth, UUID in URL, JSON blob payloads

---

## Quick start

```bash
# 0) In the backend folder, create a .env file (see below)
cp .env.example .env   # if you have one, otherwise create it manually , I will send you the .env file seperatly

# 1) Run the API
cargo run

# 2) Health check (new terminal)
curl http://localhost:3005/api/health
# -> ok

```


# Take-home task: Client Onboarding Form

## Background

At Meela, we help match clients to therapists, but first clients need to complete comprehensive intake forms. These
forms are complex with multiple sections and sensitive mental health questions requiring careful UX.

The forms can take some time to get through. The reality is that sometimes life will demand immediate attention from our
users such that they do not complete the form in a single sitting and they would benefit from having a partial form
submission to return to. And that is the crux of this task.

## Objective

Build a _simple_ client intake form system that supports **partial form submission** with the ability to resume later.
Like, really simple. Proof-of-concept level. Don't worry about edge-cases or validation. Visit [our
site](https://app.meelahealth.com) and cherry pick a small amount of different questions that make up your form.

**The main goal**: A user should be able to fill out part of a form, save their progress, and return later to continue
where they left off.

Dos:

1. **Fork our repo!**
2. **Multi-step form** at least 3 questions.
3. **Save progress** - user can save and exit at any point - this can happen automatically, using a timer, or a button.
   All are fine.
4. **Resume capability** - user can return and continue from where they left off

Don'ts:

1. **No auth required** - having a UUID in a URL is super-good enough!
2. **No versioning required** - don't worry about handling form schema changes.
3. **No i18n!** - overkill!

**Time Estimate:** spend _max_ 4-6 hours, please.

If you do not finish in this time, stop! We can talk about what you did manage to accomplish in that time!

## Technology Stack

**Frontend**: Use React or Solid.js (your choice)

**Backend**: Feel free to use the provided Rust code in the `/backend` directory - or butcher it and take what you need.
We do take into account your prior experience with Rust so if you don't have any: do not worry, we will adapt our
evaluation accordingly.

**Database**: Must use a database (relational or NoSQL - your choice really and then you might have to make some other
choices than what has been made in the `/backend` directory)

**API**: GraphQL or REST (your choice)

**Requirements**:

- Frontend must communicate with a backend
- Backend must persist data to a database
  - That means you should **not** save the partial submissions using `localStorage` in the browser!

## Deliverables

1. Make the thing work
2. Tell us how to set it up and run it so we can review it:
   1. Either write a `.txt`/`.md` file that tells us which invocations of `cargo run`, `npm run dev`, `yarn dev`, ... we have to use, or
   2. simply provide a `justfile`/`Makefile` for us.
3. Send us a link to your fork on your github account!

## GO GO GO! 🌶️🌶️🌶️

Remember, focus on the core "resume" functionality - that is what we are evaluating. We look forward to catching up and
reviewing your submission!



