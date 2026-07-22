# FocusFlow Todo

A dependency-free full-stack Todo web app. The browser UI and Python API are
served from one process; tasks persist in SQLite.

## Run

```bash
python3 -m backend.server
```

Then open [http://127.0.0.1:8080](http://127.0.0.1:8080). The database is
created at `backend/data/todos.sqlite3` on first start.

To keep data elsewhere or choose a port:

```bash
python3 -m backend.server --port 3000 --db /tmp/focusflow.sqlite3
```

## Test

```bash
python3 -m unittest discover -s backend/tests -v
```

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/api/health` | Service health check |
| `GET` | `/api/todos` | List tasks |
| `POST` | `/api/todos` | Create a task |
| `PATCH` | `/api/todos/:id` | Update title, priority, due date, or completion |
| `DELETE` | `/api/todos/:id` | Delete a task |
