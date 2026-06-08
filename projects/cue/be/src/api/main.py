"""Cue API application."""

import json
from pathlib import Path

try:
    from fastapi import FastAPI
    from fastapi.middleware.cors import CORSMiddleware
    from fastapi.staticfiles import StaticFiles
except ImportError:
    class FastAPI:
        """Tiny route registry used by local smoke tests when FastAPI is absent."""

        def __init__(self, **_kwargs):
            self._handlers = {}

        def add_middleware(self, *_args, **_kwargs):
            return None

        def mount(self, *_args, **_kwargs):
            return None

        def get(self, path: str):
            def wrapper(fn):
                self._handlers[("GET", path)] = fn
                return fn

            return wrapper

        def post(self, path: str):
            def wrapper(fn):
                self._handlers[("POST", path)] = fn
                return fn

            return wrapper

    class CORSMiddleware:
        pass

    class StaticFiles:
        def __init__(self, *_args, **_kwargs):
            return None

try:
    from .generation import GENERATOR_ENV, get_generator
    from .fixture_store import (
        create_session,
        get_project,
        get_session,
        list_artifacts,
        list_projects,
        list_sessions,
        list_workitems,
        post_message,
        run_artifact,
        workitem_context,
    )
except ImportError:
    from generation import GENERATOR_ENV, get_generator
    from fixture_store import (
        create_session,
        get_project,
        get_session,
        list_artifacts,
        list_projects,
        list_sessions,
        list_workitems,
        post_message,
        run_artifact,
        workitem_context,
    )

CUE_ROOT = Path(__file__).resolve().parents[3]
BE_ROOT = Path(__file__).resolve().parents[2]
EXAMPLE_SPEC_PATH = CUE_ROOT / "examples" / "tracker-app-spec.v0.json"

app = FastAPI(
    title="Cue API",
    description="Prompt-to-Governed-App backend",
    version="0.1.0",
    docs_url="/api/docs",
    redoc_url="/api/redoc",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:3212",
        "http://127.0.0.1:3212",
        "http://localhost:3214",
        "http://127.0.0.1:3214",
        "http://localhost:3210",
        "http://127.0.0.1:3210",
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/api/health")
async def api_health():
    """API health endpoint."""
    return {
        "status": "healthy",
        "service": "cue-api",
        "backend": "python-reference",
        "persistence": {
            "local": "postgresql",
            "managed": "alloydb",
            "driver": "sqlalchemy+asyncpg",
            "active_store": "fixture",
        },
        "generation": {
            "provider": "deterministic",
            "env": GENERATOR_ENV,
        },
        "frontend": "vite-reference",
    }


@app.get("/health")
async def health():
    """Deployment health endpoint."""
    return await api_health()


@app.get("/api/app-spec/example")
async def get_example_app_spec():
    """Return the tracker app-spec example used by the first Cue vertical."""
    with EXAMPLE_SPEC_PATH.open("r", encoding="utf-8") as handle:
        return json.load(handle)


@app.get("/api/projects")
async def api_projects():
    """Return project-owned workstream state for Artifact Studio."""
    return list_projects()


@app.get("/api/projects/{project_id}")
async def api_project(project_id: str):
    """Return one project with sessions, WorkItems, artifacts, and stages."""
    return get_project(project_id)


@app.get("/api/projects/{project_id}/sessions")
async def api_project_sessions(project_id: str):
    """Return all sessions under a project."""
    return list_sessions(project_id)


@app.post("/api/projects/{project_id}/sessions")
async def api_create_session(project_id: str, payload: dict):
    """Create a project-scoped session."""
    return create_session(project_id, payload)


@app.get("/api/sessions/{session_id}")
async def api_session(session_id: str):
    """Return a project-scoped session by id."""
    return get_session(session_id)


@app.post("/api/sessions/{session_id}/messages")
async def api_session_message(session_id: str, payload: dict):
    """Append a message and update WorkItem workflow state."""
    return post_message(session_id, payload)


@app.get("/api/projects/{project_id}/workitems")
async def api_project_workitems(project_id: str):
    """Return project WorkItems."""
    return list_workitems(project_id)


@app.get("/api/projects/{project_id}/artifacts")
async def api_project_artifacts(project_id: str):
    """Return project artifacts."""
    return list_artifacts(project_id)


@app.get("/api/workitems/{workitem_id}/context")
async def api_workitem_context(workitem_id: str):
    """Return the right-panel context for one WorkItem."""
    return workitem_context(workitem_id)


@app.post("/api/workitems/{workitem_id}/artifact-runs")
async def api_artifact_run(workitem_id: str, payload: dict):
    """Create or update an artifact from an accepted WorkItem."""
    return run_artifact(workitem_id, payload)


@app.post("/api/agent-team/run")
async def agent_team_run(payload: dict):
    """Run a governed agent team and return a structured artifact (#1545).

    Conforms to `projects/cue/schemas/agent-team-artifact.v0.schema.json`.
    """
    return get_generator().agent_team_artifact(
        payload.get("prompt", ""),
        payload.get("roles", []),
    )


static_path = BE_ROOT / "static"
if static_path.exists() and static_path.is_dir():
    app.mount("/", StaticFiles(directory=str(static_path), html=True), name="static")
