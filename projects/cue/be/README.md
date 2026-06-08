# Cue Backend

Cue backend code currently uses a Python reference API while the cclab runtime
path catches up.

This scaffold keeps the first backend slice intentionally small:

- `GET /api/health` reports service status.
- `GET /api/app-spec/example` returns `../examples/tracker-app-spec.v0.json`.
- `src/db/` contains the PostgreSQL reference implementation using
  SQLAlchemy async + asyncpg. The running API still defaults to fixture-backed
  storage until the runtime request/session lifecycle is ready.
- `static/` is the frontend build output mount point.
- `src/api/generation.py` is the generation provider boundary. Local tests use
  the deterministic provider and never call an LLM.

Persistence target:

- Local development: PostgreSQL.
- Managed runtime: AlloyDB.
- Reference Python stack: SQLAlchemy async engine + asyncpg.
- Schema: `src/db/schema.py`; Alembic-compatible migration:
  `src/db/migrations/001_control_plane.py`.

Generation target:

- Local development and e2e: `CUE_LLM_PROVIDER=deterministic` (default).
- Runtime LLM providers must implement the same provider surface before they
  are wired into `/api/sessions/{session_id}/messages`,
  `/api/workitems/{workitem_id}/artifact-runs`, or `/api/agent-team/run`.
- API and DB tests should stay provider-deterministic; network-backed LLM tests
  belong in a separate integration profile.

Run target:

```bash
python main.py
```

Mamba remains the parity target after the reference API behavior is stable.
