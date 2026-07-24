# FocusFlow Todo

A full-stack Todo web app. A Rust (Axum) API serves the browser UI from the
same process; tasks persist in SQLite through `rusqlite`.

## Run

```bash
cargo run --manifest-path backend-rust/Cargo.toml
```

Then open [http://127.0.0.1:8080](http://127.0.0.1:8080). The database is
created at `backend-rust/data/todos.sqlite3` on first start.

To keep data elsewhere or choose a port:

```bash
cargo run --manifest-path backend-rust/Cargo.toml -- --port 3000 --db /tmp/focusflow.sqlite3
```

## Test

```bash
cargo test --manifest-path backend-rust/Cargo.toml
```

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/api/health` | Service health check |
| `GET` | `/api/todos` | List tasks |
| `POST` | `/api/todos` | Create a task |
| `PATCH` | `/api/todos/:id` | Update title, priority, due date, or completion |
| `DELETE` | `/api/todos/:id` | Delete a task |
