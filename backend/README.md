# Backend template

## Setup

1. Install sqlx cli

    ```sh
    $ cargo install sqlx-cli
    ```

2. Create the database.

    ```sh
    $ sqlx db create
    ```

3. Run sql migrations

    ```sh
    $ sqlx migrate run
    ```

## Usage

Start the server

```
cargo run



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
