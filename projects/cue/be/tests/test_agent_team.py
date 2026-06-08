"""Smoke tests for the Cue reference API.

The reference implementation uses FastAPI when installed. `main.py` also
contains tiny fallbacks for local smoke tests so these assertions can run in
plain CPython without booting a server.

Run:  pytest projects/cue/be/tests/test_agent_team.py -v
"""

import asyncio
import sys


# ── Tests ────────────────────────────────────────────────────────────────────

REQUIRED_KEYS = {
    "requirements_summary",
    "app_spec_changes",
    "implementation",
    "tests",
    "release_package",
    "review_tickets",
}


def _load_main_app():
    sys.modules.pop("main", None)
    sys.modules.pop("fixture_store", None)
    src_dir = (
        # projects/cue/be/src/api/main.py — make the api package importable
        # via just `main` after putting its parent on sys.path.
        __import__("pathlib").Path(__file__).resolve().parents[1] / "src" / "api"
    )
    if str(src_dir) not in sys.path:
        sys.path.insert(0, str(src_dir))
    import main  # type: ignore[import-not-found]
    return main


def test_agent_team_run_returns_artifact_with_six_required_keys() -> None:
    main = _load_main_app()

    payload = {"prompt": "hi", "roles": ["pm"]}
    artifact = asyncio.run(main.agent_team_run(payload))

    assert isinstance(artifact, dict), "endpoint must return a dict"
    missing = REQUIRED_KEYS - set(artifact.keys())
    assert not missing, f"missing required artifact keys: {missing}"
    assert artifact.get("schema_version") == "cue.agent-team-artifact.v0", (
        "schema_version must be coerced to the v0 const"
    )


def test_agent_team_run_threads_roles_through_to_team_run() -> None:
    main = _load_main_app()

    payload = {"prompt": "ping", "roles": ["pm", "dev", "qa"]}
    artifact = asyncio.run(main.agent_team_run(payload))

    # T4 stub echoes roles + prompt — sanity check that the wire-up
    # passed the payload all the way through to team_run.
    assert artifact["roles"] == ["pm", "dev", "qa"]
    assert artifact["prompt"] == "ping"


def test_health_reports_postgres_persistence_target() -> None:
    main = _load_main_app()

    payload = asyncio.run(main.api_health())

    assert payload["persistence"] == {
        "local": "postgresql",
        "managed": "alloydb",
        "driver": "sqlalchemy+asyncpg",
        "active_store": "fixture",
    }
    assert payload["backend"] == "python-reference"
    assert payload["frontend"] == "vite-reference"
    assert payload["generation"] == {
        "provider": "deterministic",
        "env": "CUE_LLM_PROVIDER",
    }


def test_projects_api_returns_project_scoped_workstream_state() -> None:
    main = _load_main_app()

    payload = asyncio.run(main.api_projects())

    assert len(payload["projects"]) >= 2
    project = payload["projects"][0]
    assert project["id"] == "team-request-tracker"
    assert project["sessions"][0]["project_id"] == project["id"]
    assert project["workitems"][0]["project_id"] == project["id"]
    assert project["workitems"][0]["qc_status"] == "pass"
    assert project["artifacts"][0]["workitem_id"] == "request-tracker-prd"


def test_general_chat_message_redirects_without_creating_workitem() -> None:
    main = _load_main_app()

    response = asyncio.run(
        main.api_session_message(
            "session-request-tracker",
            {"content": "你好，今天天氣如何？"},
        )
    )

    assert response["classification"] == "general_chat_redirect"
    assert response["context"]["type"] == "project_overview"
    assert len(response["project"]["workitems"]) == 2


def test_website_prompt_creates_workitem_with_prd_td_website_plan() -> None:
    main = _load_main_app()

    response = asyncio.run(
        main.api_session_message(
            "session-request-tracker",
            {"content": "幫我產生一個 marketing 網站，收集 lead 並追蹤成效。"},
        )
    )

    workitem = response["workitem"]
    assert response["classification"] == "project_work"
    assert workitem["target"] == "Website"
    assert [step["id"] for step in workitem["workflow_plan"]] == ["prd", "td", "website"]
    assert workitem["qc_status"] == "pass"
    assert response["context"]["next_artifact_kind"] == "prd"


def test_artifact_run_rejects_unaccepted_workitem() -> None:
    main = _load_main_app()

    response = asyncio.run(
        main.api_artifact_run(
            "weekly-report-intake",
            {"kind": "prd"},
        )
    )

    assert response["status"] == "rejected"
    assert response["reason"] == "workitem_not_accepted"


def test_artifact_run_creates_prd_for_accepted_workitem() -> None:
    main = _load_main_app()

    response = asyncio.run(
        main.api_artifact_run(
            "request-tracker-prd",
            {"kind": "prd"},
        )
    )

    assert response["status"] == "created"
    assert any(
        artifact["id"] == "request-tracker-prd-prd-v1"
        for artifact in response["project"]["artifacts"]
    )
    assert response["qc_result"]["status"] == "pending"
    assert response["context"]["next_artifact_kind"] == "td"


class _FakeAlembicOp:
    def __init__(self) -> None:
        self.sql: list[str] = []

    def execute(self, sql: str) -> None:
        self.sql.append(sql)


def _load_python_module(module_name: str, path_parts: list[str]):
    base_path = __import__("pathlib").Path(__file__).resolve().parents[1]
    path = base_path.joinpath(*path_parts)
    spec = __import__("importlib.util").util.spec_from_file_location(
        module_name,
        path,
    )
    module = __import__("importlib.util").util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_control_plane_schema_creates_pg_tables() -> None:
    schema = _load_python_module("cue_control_plane_schema", ["src", "db", "schema.py"])

    sql = "\n".join(schema.CONTROL_PLANE_UP_SQL)

    assert "CREATE TABLE IF NOT EXISTS cue_projects" in sql
    assert "CREATE TABLE IF NOT EXISTS cue_sessions" in sql
    assert "CREATE TABLE IF NOT EXISTS cue_messages" in sql
    assert "CREATE TABLE IF NOT EXISTS cue_workitems" in sql
    assert "CREATE TABLE IF NOT EXISTS cue_artifacts" in sql
    assert "CREATE TABLE IF NOT EXISTS cue_artifact_versions" in sql
    assert "CREATE TABLE IF NOT EXISTS cue_qc_runs" in sql
    assert "CREATE TABLE IF NOT EXISTS cue_qc_checks" in sql
    assert "workflow_plan   JSONB NOT NULL DEFAULT '[]'" in sql
    assert "idx_cue_qc_runs_target" in sql


def test_control_plane_migration_is_alembic_compatible() -> None:
    src_dir = __import__("pathlib").Path(__file__).resolve().parents[1] / "src"
    if str(src_dir) not in sys.path:
        sys.path.insert(0, str(src_dir))
    migration = _load_python_module(
        "cue_control_plane_migration",
        ["src", "db", "migrations", "001_control_plane.py"],
    )

    op = _FakeAlembicOp()
    migration.upgrade(op)

    assert migration.revision == "001_control_plane"
    assert any("CREATE TABLE IF NOT EXISTS cue_workitems" in statement for statement in op.sql)
