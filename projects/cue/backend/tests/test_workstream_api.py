"""Contract tests for the Cue Artifact Studio workstream API."""

from __future__ import annotations

import asyncio
import json
import importlib
import subprocess
import sys
import types
from pathlib import Path


def _install_httpkit_stub() -> None:
    try:
        importlib.import_module("mambalibs.http")
        return
    except ImportError:
        pass

    if "mambalibs" not in sys.modules:
        pkg = types.ModuleType("mambalibs")
        pkg.__path__ = []
        sys.modules["mambalibs"] = pkg

    api_mod = types.ModuleType("mambalibs.http")

    class App:
        def __init__(self, **_kwargs) -> None:
            self._handlers: dict[tuple[str, str], object] = {}

        def add_middleware(self, *_args, **_kwargs) -> None:
            return None

        def mount(self, *_args, **_kwargs) -> None:
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
        def __init__(self, *_args, **_kwargs) -> None:
            return None

    api_mod.App = App
    api_mod.FastAPI = App
    api_mod.CORSMiddleware = CORSMiddleware
    api_mod.StaticFiles = StaticFiles
    sys.modules["mambalibs.http"] = api_mod


def _load_main():
    src_dir = Path(__file__).resolve().parents[1] / "src"
    if str(src_dir) not in sys.path:
        sys.path.insert(0, str(src_dir))
    _install_httpkit_stub()
    for module_name in ["main", "services", "store", "models"]:
        sys.modules.pop(module_name, None)
    import main  # type: ignore[import-not-found]

    return main


def test_project_reads_return_workstream_payload() -> None:
    main = _load_main()
    handler = main.app._handlers[("GET", "/api/projects/{project_id}")]

    payload = asyncio.run(handler("team-request-tracker"))

    assert payload["ok"] is True
    assert payload["data"]["id"] == "team-request-tracker"
    assert payload["data"]["workitems"][0]["state"] == "accepted"
    assert payload["data"]["artifacts"][0]["versions"][0]["body"]["goal"]
    assert payload["data"]["active_session_id"] == "session-team-request-tracker"
    assert payload["data"]["sessions"][0]["messages"]


def test_project_list_uses_artifact_studio_client_shape() -> None:
    main = _load_main()
    handler = main.app._handlers[("GET", "/api/projects")]

    payload = asyncio.run(handler())

    assert "projects" in payload
    project = payload["projects"][0]
    assert project["id"] == "team-request-tracker"
    assert project["active_session_id"] == "session-team-request-tracker"
    assert project["sessions"][0]["id"] == "session-team-request-tracker"
    assert project["workitems"][0]["workflow_plan"][0]["id"] == "prd"
    assert project["workitems"][0]["workflow_plan"][0]["agent_role"] == "pm"
    assert project["workitems"][0]["workflow_plan"][1]["agent_role"] == "architect"


def test_project_registry_exposes_hidden_repo_runtime_and_ownership_boundaries() -> None:
    main = _load_main()
    handler = main.app._handlers[("GET", "/api/admin/projects/{project_id}/registry")]

    payload = asyncio.run(handler("team-request-tracker"))

    assert payload["ok"] is True
    registry = payload["data"]
    assert registry["artifact_repository"]["gitlab_full_path"].endswith("t-operations-team-request-tracker")
    assert registry["artifact_repository"]["user_visible"] is False
    assert registry["ownership_namespace"]["namespace"] == "team"
    assert registry["ownership_namespace"]["gitlab_mapping"]["user_visible"] is False
    assert registry["runtime_tenants"][0]["environment"] == "sandbox"
    assert registry["runtime_tenants"][0]["storage"]["local_backend"] == "postgresql"
    assert registry["runtime_tenants"][0]["storage"]["backend"] == "postgresql"
    assert registry["user_visible_infrastructure"] is False


def test_app_registry_catalog_search_filter_and_internal_mapping() -> None:
    main = _load_main()
    catalog_handler = main.app._handlers[("POST", "/api/admin/app-registry/catalog")]
    duplicate_handler = main.app._handlers[("POST", "/api/admin/app-registry/duplicates")]

    catalog = asyncio.run(
        catalog_handler(
            {
                "query": "request",
                "filters": {"risk_tier": "tier_2", "lifecycle_status": "sandbox", "data_sources": "request_tracker_db"},
                "include_internal": True,
            }
        )
    )
    duplicates = asyncio.run(duplicate_handler({"name": "Team Request Tracker", "owner": "Operations"}))

    assert catalog["ok"] is True
    row = catalog["data"][0]
    assert row["app_id"] == "team-request-tracker"
    assert row["hidden_repo_backed"] is True
    assert row["permission_summary"]["sensitive_fields_masked"] is True
    assert row["source_mapping"]["gitlab_project_id"] == 10001
    assert row["source_mapping"]["current_commit_sha"]
    assert row["retention_behavior"] == "retain_hidden_repo_audit_history"
    assert duplicates["data"][0]["app_id"] == "team-request-tracker"


def test_owner_api_exposes_runtime_tenant_without_gitlab_identity() -> None:
    main = _load_main()
    tenant_handler = main.app._handlers[("GET", "/api/projects/{project_id}/runtime-tenants")]
    namespace_handler = main.app._handlers[("GET", "/api/projects/{project_id}/ownership-namespace")]

    tenants = asyncio.run(tenant_handler("team-request-tracker"))
    namespace = asyncio.run(namespace_handler("team-request-tracker"))

    assert tenants["ok"] is True
    assert tenants["data"][0]["runtime_families"] == [
        "record",
        "comment",
        "attachment",
        "workflow_state",
        "dashboard_materialization",
        "usage_metric",
        "runtime_audit",
    ]
    assert "gitlab_project_id" not in str(tenants["data"])
    assert namespace["ok"] is True
    assert "gitlab_mapping" not in namespace["data"]
    assert namespace["data"]["quota_policy"] == "team_default"
    assert namespace["data"]["owner"]["owner_team"] == "Operations"


def test_hidden_repo_template_manifest_matches_fixture_files() -> None:
    main = _load_main()
    handler = main.app._handlers[("GET", "/api/admin/hidden-repo-template")]

    payload = asyncio.run(handler())

    assert payload["ok"] is True
    manifest = payload["data"]
    assert manifest["user_visible"] is False
    assert manifest["mvp_generated_code_required"] is False
    template_root = Path(__file__).resolve().parents[2] / "app-repo-template"
    for relative_path in manifest["required_files"]:
        target = template_root / relative_path
        assert target.exists(), f"missing hidden repo template entry: {relative_path}"


def test_product_layout_routes_owner_admin_backend_and_legacy_paths() -> None:
    main = _load_main()
    manifest_handler = main.app._handlers[("GET", "/api/admin/product-layout")]
    route_handler = main.app._handlers[("GET", "/api/admin/product-layout/route/{audience}")]

    manifest = asyncio.run(manifest_handler())
    owner = asyncio.run(route_handler("owner"))
    operator = asyncio.run(route_handler("operator"))
    backend = asyncio.run(route_handler("backend"))
    legacy = asyncio.run(route_handler("legacy"))

    workspace_paths = {workspace["path"] for workspace in manifest["data"]["workspaces"]}
    cue_root = Path(__file__).resolve().parents[2]
    for relative_path in workspace_paths:
        assert (cue_root.parents[1] / relative_path).exists(), relative_path
    assert owner["data"]["path"] == "projects/cue/artifact-studio"
    assert operator["data"]["path"] == "projects/cue/admin"
    assert backend["data"]["path"] == "projects/cue/backend"
    assert legacy["data"]["path"] == "projects/cue/docs/legacy"
    assert not (cue_root / "Cargo.toml").exists()
    assert manifest["data"]["legacy_retirement"]["status"] == "retired"
    assert manifest["data"]["legacy_retirement"]["closed_issues"] == [1243, 1245, 1246, 1247, 1248, 1226]
    assert manifest["data"]["session_boundary"]["auth_mode"] == "placeholder"
    assert manifest["data"]["session_boundary"]["api_base_path"] == "/api"
    assert (cue_root / "docs" / "legacy" / "RETIREMENT.md").exists()
    assert (cue_root / "shared" / "session.ts").exists()


def test_app_spec_validator_accepts_template_contract() -> None:
    main = _load_main()
    template = json.loads((Path(__file__).resolve().parents[2] / "app-repo-template" / "app-spec.json").read_text())
    handler = main.app._handlers[("POST", "/api/admin/app-spec/validate")]

    payload = asyncio.run(handler({"phase": "sandbox", "spec": template}))

    assert payload["ok"] is True
    assert payload["data"]["valid"] is True
    assert payload["data"]["deployment_gate"] == "ready"
    assert payload["data"]["errors"] == []


def test_app_spec_validator_blocks_structured_invalid_designs() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/admin/app-spec/validate")]

    payload = asyncio.run(
        handler(
            {
                "phase": "production",
                "spec": {
                    "schema_version": "cue.app-spec.v0",
                    "app_id": "risky-app",
                    "name": "Risky App",
                    "owner_team": "Ops",
                    "owner_user": "owner@example.com",
                    "lifecycle_status": "draft",
                    "permissions": {"roles": {"everyone": ["read", "write"]}},
                    "workflow": {
                        "states": ["draft", "approved"],
                        "transitions": [{"from": "draft", "to": "missing"}],
                    },
                },
            }
        )
    )

    assert payload["ok"] is True
    assert payload["data"]["valid"] is False
    assert payload["data"]["deployment_gate"] == "blocked"
    codes = {error["code"] for error in payload["data"]["errors"]}
    assert {"everyone_edit_access", "impossible_transition"} <= codes


def test_connector_catalog_and_policy_checks_gate_app_specs() -> None:
    main = _load_main()
    catalog_handler = main.app._handlers[("GET", "/api/admin/connectors")]
    validate_handler = main.app._handlers[("POST", "/api/admin/app-spec/validate")]

    catalog = asyncio.run(catalog_handler())
    assert catalog["ok"] is True
    assert catalog["data"]["connectors"]["request_tracker_db"]["mode"] == "read_only"
    assert "field" in catalog["data"]["connectors"]["request_tracker_db"]["scopes"]

    payload = asyncio.run(
        validate_handler(
            {
                "phase": "sandbox",
                "spec": {
                    "schema_version": "cue.app-spec.v0",
                    "app_id": "connector-app",
                    "name": "Connector App",
                    "owner_team": "Ops",
                    "owner_user": "owner@example.com",
                    "lifecycle_status": "draft",
                    "connectors": [
                        {"id": "unknown_source", "data_owner": "ops@example.com", "fields": ["email"]},
                        {"id": "request_tracker_db", "data_owner": "ops@example.com", "fields": ["amount", "ssn"]},
                    ],
                },
            }
        )
    )

    codes = {error["code"] for error in payload["data"]["errors"]}
    assert "unregistered_connector" in codes
    assert "connector_field_not_allowed" in codes
    assert "sensitive_field_requires_mask_or_approval" in codes


def test_permission_simulation_reports_field_and_action_access() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/admin/app-spec/permissions/simulate")]

    payload = asyncio.run(
        handler(
            {
                "role": "editor",
                "spec": {
                    "permissions": {"roles": {"editor": ["read", "write"]}},
                    "entities": [
                        {
                            "id": "request",
                            "fields": [
                                {"id": "status", "sensitivity": "normal"},
                                {"id": "amount", "sensitivity": "sensitive"},
                            ],
                        }
                    ],
                },
            }
        )
    )

    assert payload["ok"] is True
    assert payload["data"]["allowed_actions"] == ["read", "write"]
    amount = [field for field in payload["data"]["field_access"] if field["field"] == "amount"][0]
    assert amount["visible"] is True
    assert amount["editable"] is True
    assert amount["masked"] is True


def test_policy_risk_evaluator_covers_tier_zero_through_tier_four() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/admin/app-spec/policy-risk")]

    cases = [
        ({"entities": []}, "tier_0", "pass"),
        ({"entities": [{"id": "task", "fields": []}]}, "tier_1", "pass"),
        (
            {
                "entities": [{"id": "task", "fields": []}],
                "connectors": [{"id": "request_tracker_db", "data_owner": "ops@example.com", "mode": "read_only"}],
            },
            "tier_2",
            "pass",
        ),
        (
            {
                "entities": [{"id": "task", "fields": []}],
                "connectors": [{"id": "request_tracker_db", "data_owner": "ops@example.com", "mode": "read_only"}],
                "restricted_export": True,
                "permissions": {"roles": {"manager": ["read"]}},
                "risk_tier": "tier_3",
            },
            "tier_3",
            "warning",
        ),
        (
            {
                "entities": [{"id": "task", "fields": []}],
                "actions": ["price_change"],
                "permissions": {"roles": {"everyone": ["read", "write"]}},
            },
            "tier_4",
            "block",
        ),
    ]

    for spec, expected_tier, expected_result in cases:
        payload = asyncio.run(handler({"spec": spec}))
        assert payload["ok"] is True
        assert payload["data"]["risk_tier"] == expected_tier
        assert payload["data"]["policy"]["result"] == expected_result


def test_policy_risk_evaluator_blocks_direct_db_and_tier_four_self_service() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/admin/app-spec/policy-risk")]

    payload = asyncio.run(
        handler(
            {
                "spec": {
                    "risk_tier": "tier_4",
                    "connectors": [{"id": "raw-db", "mode": "direct_database", "data_owner": "ops@example.com"}],
                }
            }
        )
    )

    codes = {block["code"] for block in payload["data"]["policy"]["blocks"]}
    assert payload["data"]["policy"]["sandbox_allowed"] is False
    assert "direct_db_access" in codes
    assert "tier_4_self_service_blocked" in codes


def test_app_spec_diff_reports_governed_change_categories() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/admin/app-spec/diff")]

    payload = asyncio.run(
        handler(
            {
                "old": {
                    "fields": [{"id": "amount", "type": "number"}],
                    "permissions": {"roles": {"viewer": ["read"]}},
                    "risk_tier": "tier_1",
                },
                "new": {
                    "fields": [{"id": "amount", "type": "currency"}],
                    "permissions": {"roles": {"editor": ["read", "write"]}},
                    "risk_tier": "tier_2",
                    "automations": [{"id": "notify-finance"}],
                },
            }
        )
    )

    assert payload["ok"] is True
    assert payload["data"]["changed"] is True
    assert payload["data"]["categories"]["fields"][0]["path"] == "fields"
    assert payload["data"]["categories"]["permissions"][0]["path"] == "permissions"
    assert payload["data"]["categories"]["automation"][0]["path"] == "automations"
    assert payload["data"]["categories"]["risk"][0]["path"] == "risk_tier"


def test_app_spec_preview_and_controlled_editor_gate_sandbox() -> None:
    main = _load_main()
    preview_handler = main.app._handlers[("POST", "/api/admin/app-spec/preview")]
    edit_handler = main.app._handlers[("POST", "/api/admin/app-spec/controlled-edit")]
    old_spec = {
        "schema_version": "cue.app-spec.v0",
        "app_id": "tracker",
        "name": "Tracker",
        "owner_team": "Operations",
        "owner_user": "ops@example.com",
        "lifecycle_status": "draft",
        "entities": [{"id": "request", "fields": [{"id": "status", "sensitivity": "normal"}]}],
        "permissions": {"roles": {"owner": ["read", "write"]}},
        "workflow": {"states": ["new", "done"], "transitions": [{"from": "new", "to": "done"}]},
    }

    preview = asyncio.run(preview_handler({"spec": old_spec}))
    accepted = asyncio.run(edit_handler({"old": old_spec, "edit": {"path": "permissions", "value": {"roles": {"owner": ["read"]}}}}))
    rejected = asyncio.run(edit_handler({"old": old_spec, "edit": {"path": "entities", "value": []}}))

    assert preview["data"]["form_preview"][0]["entity"] == "request"
    assert preview["data"]["table_preview"][0]["columns"] == ["status"]
    assert accepted["data"]["status"] == "accepted"
    assert accepted["data"]["diff"]["categories"]["permissions"][0]["path"] == "permissions"
    assert accepted["data"]["sandbox_deployment"]["allowed"] is True
    assert rejected["data"]["status"] == "rejected"
    assert rejected["data"]["reason"] == "unsupported_edit_path"


def test_tracker_primitive_renderer_outputs_role_aware_runtime_views() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/admin/tracker/render-primitives")]

    payload = asyncio.run(
        handler(
            {
                "role": "viewer",
                "spec": {
                    "permissions": {"roles": {"viewer": ["read"]}},
                    "entities": [
                        {
                            "id": "request",
                            "fields": [
                                {"id": "status", "type": "status", "required": True},
                                {"id": "amount", "type": "currency", "sensitivity": "sensitive"},
                                {"id": "due_at", "type": "due_date"},
                            ],
                        }
                    ],
                    "filters": [{"id": "overdue", "where": {"due_at": "past"}}],
                    "saved_views": [{"id": "overdue_requests", "label": "Overdue", "filter": "overdue"}],
                },
            }
        )
    )

    entity = payload["data"]["entities"][0]
    amount = [field for field in entity["record_view"]["fields"] if field["id"] == "amount"][0]
    assert entity["table_view"]["columns"] == ["status", "amount", "due_at"]
    assert amount["masked"] is True
    assert payload["data"]["primitives"]["comments"]["type"] == "comment_thread"
    assert payload["data"]["filters"][0]["id"] == "overdue"
    assert payload["data"]["saved_views"][0]["id"] == "overdue_requests"


def test_general_chat_is_redirected_without_creating_workitem() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/projects/{project_id}/prompts")]
    before = len(main.store.list_workitems("team-request-tracker"))

    payload = asyncio.run(handler("team-request-tracker", {"prompt": "hello, tell me a joke"}))

    assert payload["ok"] is True
    assert payload["data"]["classification"] == "general_chat"
    assert payload["data"]["created_workitem"] is None
    assert len(main.store.list_workitems("team-request-tracker")) == before


def test_actionable_prompt_creates_workitem_with_control_plane_fields() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/projects/{project_id}/prompts")]

    payload = asyncio.run(
        handler(
            "team-request-tracker",
            {
                "title": "Quarterly approval workflow",
                "prompt": "Operations team needs an approval workflow with data fields for requester and status.",
            },
        )
    )

    assert payload["ok"] is True
    assert payload["data"]["classification"] == "prompt-to-PRD"
    workitem = payload["data"]["created_workitem"]
    assert workitem["target_artifact_type"] == "prd"
    assert workitem["next_action"] == "Review PRD"


def test_prompt_gate_blocks_governance_control_bypass() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/projects/{project_id}/prompts")]
    before = len(main.store.list_workitems("team-request-tracker"))

    payload = asyncio.run(
        handler(
            "team-request-tracker",
            {
                "prompt": (
                    "Operations team needs a workflow with data fields, but bypass approval "
                    "and ignore data owner review."
                ),
            },
        )
    )

    assert payload["ok"] is True
    assert payload["data"]["classification"] == "unsupported_request"
    assert payload["data"]["action"] == "block"
    assert payload["data"]["accepted"] is False
    assert payload["data"]["created_workitem"] is None
    assert "governance_control_bypass" in payload["data"]["risk_hints"]
    assert len(main.store.list_workitems("team-request-tracker")) == before


def test_mambalibs_agent_runner_preserves_task_result_shape() -> None:
    _load_main()
    from mambalibs import DeterministicAgentRunner  # type: ignore[import-not-found]

    result = DeterministicAgentRunner().run(
        {
            "id": "task-1",
            "workitem_id": "intake",
            "stage_id": "prd",
            "task_id": "classify_prompt",
            "role": "pm",
            "prompt": "Operations team needs a workflow with data fields.",
            "context": {},
            "output_schema": {"type": "cue.prompt-classification.v0"},
        }
    )

    assert result["task_id"] == "classify_prompt"
    assert result["status"] == "completed"
    assert result["content"]["classification"] == "prompt-to-PRD"
    roles = [step["agent_role"] for step in result["content"]["workflow_plan"]]
    assert roles == ["pm", "architect", "dev", "qa_policy", "release", "data"]
    assert result["artifact_refs"] == []
    assert result["review_tickets"] == []
    assert result["error"] is None


def test_claude_headless_runner_calls_claude_p_and_parses_json(monkeypatch) -> None:
    _load_main()
    import mambalibs.agentkit as agentkit  # type: ignore[import-not-found]

    calls = []

    def fake_run(args, **kwargs):
        calls.append((args, kwargs))
        return subprocess.CompletedProcess(
            args,
            0,
            stdout=(
                '{"task_id":"classify_prompt","status":"completed",'
                '"content":{"classification":"prompt-to-PRD","action":"create_workitem",'
                '"accepted":true,"missing_fields":[]},"artifact_refs":[],"review_tickets":[],"error":null}'
            ),
            stderr="",
        )

    monkeypatch.setattr(agentkit.subprocess, "run", fake_run)
    result = agentkit.ClaudeHeadlessAgentRunner(command="claude", timeout_seconds=5).run(
        {
            "id": "task-claude",
            "workitem_id": "intake",
            "stage_id": "prd",
            "task_id": "classify_prompt",
            "role": "pm",
            "prompt": "Operations team needs a workflow with data fields.",
            "context": {},
            "output_schema": {"type": "cue.prompt-classification.v0"},
        }
    )

    assert calls[0][0][0:2] == ["claude", "-p"]
    assert "Operations team needs a workflow" in calls[0][0][2]
    assert calls[0][1]["timeout"] == 5
    assert result["status"] == "completed"
    assert result["content"]["classification"] == "prompt-to-PRD"
    assert result["content"]["workflow_plan"][-1]["agent_role"] == "data"


def test_mambalibs_pgkit_marks_postgres_preview_unavailable() -> None:
    _load_main()
    from mambalibs import UnavailablePgKit  # type: ignore[import-not-found]

    result = UnavailablePgKit({"schema": "cue"}).execute(
        {"kind": "select", "sql": "select 1", "params": {}}
    )

    assert result["status"] == "unavailable"
    assert result["rows"] == []
    assert result["row_count"] == 0
    assert "pgkit" in str(result["error"])


def test_prompt_flow_records_pgkit_commands() -> None:
    main = _load_main()
    before = len(main.store.pgkit.commands)
    handler = main.app._handlers[("POST", "/api/sessions/{session_id}/messages")]

    asyncio.run(
        handler(
            "session-team-request-tracker",
            {"content": "Operations team needs a purchase approval workflow with data fields."},
        )
    )

    commands = main.store.pgkit.commands[before:]
    sql = "\n".join(command["sql"] for command in commands)
    assert "insert into cue_session_messages" in sql
    assert "insert into cue_workitems" in sql
    assert "insert into cue_audit_events" in sql


def test_session_message_creates_workitem_for_artifact_studio() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/sessions/{session_id}/messages")]

    payload = asyncio.run(
        handler(
            "session-team-request-tracker",
            {"content": "Operations team needs a vendor intake workflow with data fields for requester and status."},
        )
    )

    assert payload["classification"] == "project_work"
    assert payload["project"]["id"] == "team-request-tracker"
    assert payload["session"]["id"] == "session-team-request-tracker"
    assert payload["session"]["messages"][-2]["speaker"] == "owner"
    assert payload["session"]["messages"][-1]["speaker"] == "cue"
    assert payload["workitem"]["target_artifact_type"] == "prd"
    assert payload["context"]["next_artifact_kind"] == "prd"
    assert payload["context"]["workflow_plan"][0]["state"] == "ready"
    assert payload["context"]["workflow_plan"][0]["agent_label"] == "PM agent"
    assert payload["context"]["workflow_plan"][-1]["agent_label"] == "Data agent"


def test_session_message_redirects_general_chat() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/sessions/{session_id}/messages")]
    before = len(main.store.list_workitems("team-request-tracker"))

    payload = asyncio.run(handler("session-team-request-tracker", {"content": "hello, tell me a joke"}))

    assert payload["classification"] == "general_chat_redirect"
    assert payload["context"]["type"] == "project_overview"
    assert len(main.store.list_workitems("team-request-tracker")) == before


def test_workitem_context_exposes_prompt_to_workitem_state() -> None:
    main = _load_main()
    handler = main.app._handlers[("GET", "/api/workitems/{workitem_id}/context")]

    payload = asyncio.run(handler("request-tracker-retention"))

    assert payload["type"] == "blockers"
    assert payload["workitem"]["id"] == "request-tracker-retention"
    assert payload["qc_status"] == "needs_input"
    assert payload["workflow_plan"][0]["state"] == "blocked"
    assert payload["workflow_plan"][0]["agent_role"] == "pm"


def test_artifact_graph_and_dependency_gates_lock_downstream_artifacts() -> None:
    main = _load_main()
    graph_handler = main.app._handlers[("GET", "/api/admin/artifact-graph")]
    context_handler = main.app._handlers[("GET", "/api/workitems/{workitem_id}/context")]

    graph = asyncio.run(graph_handler())
    accepted = asyncio.run(context_handler("request-tracker-prd"))
    collecting = asyncio.run(context_handler("request-tracker-retention"))

    assert graph["data"]["dependencies"][0] == {"from": "workitem", "to": "prd", "gate": "accepted"}
    accepted_gates = {gate["kind"]: gate for gate in accepted["artifact_gates"]}
    collecting_gates = {gate["kind"]: gate for gate in collecting["artifact_gates"]}
    assert accepted_gates["prd"]["unlocked"] is True
    assert accepted_gates["td"]["unlocked"] is False
    assert accepted_gates["runtime_manifest"]["unlocked"] is False
    assert collecting_gates["prd"]["unlocked"] is False


def test_prd_creation_rejects_collecting_workitem() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/workitems/{workitem_id}/prd")]

    payload = asyncio.run(handler("request-tracker-retention", {"summary": "Should not be created"}))

    assert payload["ok"] is False
    assert payload["error"] == "workitem_not_accepted"


def test_accepted_workitem_can_create_prd_version() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/workitems/{workitem_id}/prd")]

    payload = asyncio.run(handler("request-tracker-prd", {"summary": "Updated PRD"}))

    assert payload["ok"] is True
    assert payload["data"]["workitem"]["state"] == "drafting"
    version = payload["data"]["artifact"]["versions"][-1]
    assert version["summary"] == "Updated PRD"
    assert version["body"]["source_workitem_id"] == "request-tracker-prd"
    assert "accepted WorkItem" in version["body"]["explanation"]
    assert version["body"]["downstream_prerequisites"] == [
        "owner_review",
        "td_approval",
        "sandbox_release_evidence",
    ]


def test_admin_evidence_includes_audit_events() -> None:
    main = _load_main()
    handler = main.app._handlers[("GET", "/api/admin/workitems/{workitem_id}/evidence")]

    payload = asyncio.run(handler("request-tracker-prd"))

    assert payload["ok"] is True
    assert payload["data"]["workitem"]["id"] == "request-tracker-prd"
    assert payload["data"]["events"], "admin evidence must include audit trail entries"
    assert payload["data"]["artifact_repository"]["status"] == "fixture_backed"
    assert "audit-request-tracker-prd-1" in payload["data"]["audit_event_ids"]


def test_admin_evidence_exposes_blockers_and_review_tickets() -> None:
    main = _load_main()
    handler = main.app._handlers[("GET", "/api/admin/workitems/{workitem_id}/evidence")]

    payload = asyncio.run(handler("request-tracker-retention"))

    assert payload["ok"] is True
    assert payload["data"]["blockers"] == ["retention_period"]
    assert payload["data"]["review_tickets"][0]["kind"] == "backend_blocker"
    assert payload["data"]["diagnostics"]["requires_admin_review"] is True


def test_admin_operations_metrics_and_health_are_queryable() -> None:
    main = _load_main()
    audit_handler = main.app._handlers[("GET", "/api/admin/projects/{project_id}/audit")]
    metrics_handler = main.app._handlers[("GET", "/api/admin/projects/{project_id}/metrics")]
    health_handler = main.app._handlers[("GET", "/api/admin/projects/{project_id}/health-score")]

    audit = asyncio.run(audit_handler("team-request-tracker"))
    metrics = asyncio.run(metrics_handler("team-request-tracker"))
    health = asyncio.run(health_handler("team-request-tracker"))

    assert audit["ok"] is True
    assert audit["data"][0]["action"] == "classified-prompt"
    assert metrics["ok"] is True
    assert metrics["data"]["audit_event_count"] == 1
    assert metrics["data"]["last_used_at"] == "2026-05-12T10:28:00Z"
    assert metrics["data"]["workflow_trace_count"] == 1
    assert health["ok"] is True
    assert health["data"]["status"] in {"healthy", "watch", "risky"}
    assert health["data"]["metrics"]["audit_event_count"] == 1


def test_deployment_lifecycle_gates_production_rollback_and_disable() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/projects/{project_id}/deployment")]
    evidence_handler = main.app._handlers[("GET", "/api/admin/workitems/{workitem_id}/evidence")]

    blocked = asyncio.run(handler("team-request-tracker", {"action": "request_production", "actor": "ops"}))
    sandbox = asyncio.run(handler("team-request-tracker", {"action": "deploy_sandbox", "actor": "ops"}))
    request = asyncio.run(handler("team-request-tracker", {"action": "request_production", "actor": "ops"}))
    production = asyncio.run(handler("team-request-tracker", {"action": "approve_production", "actor": "ops"}))
    rollback = asyncio.run(handler("team-request-tracker", {"action": "rollback", "actor": "ops", "target_ref": "8a58d2e59c7f409184b85d9b46a0f45e3f28de8f"}))
    disabled = asyncio.run(handler("team-request-tracker", {"action": "emergency_disable", "actor": "ops"}))

    assert blocked["data"]["status"] == "blocked"
    assert sandbox["data"]["status"] == "sandbox_deployed"
    assert sandbox["data"]["gates"]["schema"] == "passed"
    assert request["data"]["review_gate"]["required_approvals"] == ["app_owner", "data_owner"]
    assert production["data"]["status"] == "production_deployed"
    assert production["data"]["release_tag"] == "cue-team-request-tracker-v1"
    assert rollback["data"]["status"] == "rolled_back"
    assert rollback["data"]["release_tag"].endswith("-rollback")
    assert disabled["data"]["status"] == "disabled"

    evidence = asyncio.run(evidence_handler("request-tracker-prd"))
    deployment_events = [event for event in evidence["data"]["events"] if event["action"].startswith("deployment-")]
    assert deployment_events[-1]["evidence"]["repo_project_id"] == 10001
    assert deployment_events[-1]["evidence"]["before"]["lifecycle_status"] == "production"
    assert deployment_events[-1]["evidence"]["after"]["lifecycle_status"] == "blocked"


def test_tracker_workflow_sandbox_enforces_sla_roles_and_notifications() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/admin/tracker/workflow-sandbox")]
    evidence_handler = main.app._handlers[("GET", "/api/admin/workitems/{workitem_id}/evidence")]

    payload = asyncio.run(
        handler(
            {
                "actor": "ops",
                "role": "owner",
                "records": [
                    {"id": "req-1", "state": "new", "transition": "start_triage", "age_hours": 2},
                    {"id": "req-2", "state": "in_progress", "transition": "resolve", "age_hours": 80},
                    {"id": "req-3", "state": "blocked", "transition": "unblock", "age_hours": 60},
                ],
            }
        )
    )

    assert payload["ok"] is True
    assert payload["data"]["transitions"][0]["to"] == "triage"
    assert payload["data"]["blocked"][0]["reason"] == "role_not_allowed"
    assert payload["data"]["blocked"][1]["reason"] == "role_not_allowed"
    notification_kinds = {notification["kind"] for notification in payload["data"]["notifications"]}
    assert {"sla_overdue", "escalation", "weekly_digest"} <= notification_kinds

    evidence = asyncio.run(evidence_handler("request-tracker-prd"))
    actions = [event["action"] for event in evidence["data"]["events"]]
    assert "workflow-state-changed" in actions
    assert "notification-scheduled" in actions


def test_tracker_regression_harness_generates_tests_and_history() -> None:
    main = _load_main()
    harness_handler = main.app._handlers[("POST", "/api/admin/tracker/regression-harness")]
    history_handler = main.app._handlers[("GET", "/api/admin/tracker/test-history")]

    payload = asyncio.run(
        harness_handler(
            {
                "app_id": "team-request-tracker",
                "app_version": 2,
                "role": "manager",
                "spec": {
                    "schema_version": "cue.app-spec.v0",
                    "app_id": "team-request-tracker",
                    "name": "Tracker",
                    "owner_team": "Operations",
                    "owner_user": "ops@example.com",
                    "lifecycle_status": "draft",
                    "entities": [{"id": "request", "fields": [{"id": "status", "sensitivity": "normal"}]}],
                    "permissions": {"roles": {"owner": ["read", "write"], "manager": ["read", "write"]}},
                },
            }
        )
    )
    history = asyncio.run(history_handler())

    test_ids = {test["id"] for test in payload["data"]["tests"]}
    assert {
        "spec-validation",
        "permission",
        "workflow-transition",
        "data-validation",
        "notification",
        "policy",
        "regression",
        "synthetic-simulation",
    } <= test_ids
    assert payload["data"]["deployment_gate"] == "passed"
    assert history["data"][0]["app_version"] == 2


def test_tracker_regression_harness_blocks_failed_p0_tests() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/admin/tracker/regression-harness")]

    payload = asyncio.run(
        handler(
            {
                "spec": {
                    "schema_version": "cue.app-spec.v0",
                    "app_id": "bad-tracker",
                    "name": "Bad Tracker",
                    "owner_team": "Operations",
                    "owner_user": "ops@example.com",
                    "lifecycle_status": "draft",
                    "actions": ["price_change"],
                    "permissions": {"roles": {"everyone": ["read", "write"]}},
                },
                "records": [{"id": "req-1", "state": "new", "transition": "resolve"}],
            }
        )
    )

    assert payload["data"]["deployment_gate"] == "blocked"
    failed_ids = {test["id"] for test in payload["data"]["p0_failed"]}
    assert {"workflow-transition", "policy"} <= failed_ids


def test_governance_transition_contract_writes_approval_registry_and_audit() -> None:
    main = _load_main()
    transition_handler = main.app._handlers[("POST", "/api/admin/projects/{project_id}/governance/transition")]
    snapshots_handler = main.app._handlers[("GET", "/api/admin/projects/{project_id}/registry-snapshots")]
    approvals_handler = main.app._handlers[("GET", "/api/admin/projects/{project_id}/approval-requests")]
    evidence_handler = main.app._handlers[("GET", "/api/admin/workitems/{workitem_id}/evidence")]

    validate = asyncio.run(
        transition_handler(
            "team-request-tracker",
            {
                "event": "validate_spec",
                "actor": "ops",
                "facts": {
                    "satisfied": [
                        "app_spec_schema_valid",
                        "owner_user_present",
                        "owner_team_present",
                        "target_users_present",
                        "at_least_one_entity",
                    ]
                },
            },
        )
    )
    sandbox = asyncio.run(
        transition_handler(
            "team-request-tracker",
            {
                "event": "deploy_sandbox",
                "actor": "ops",
                "facts": {
                    "commit_sha": "abc123",
                    "satisfied": [
                        "risk_tier_assigned",
                        "policy_check_passed",
                        "required_tests_passed",
                        "tier_4_blocklist_not_matched",
                    ],
                },
            },
        )
    )
    request = asyncio.run(transition_handler("team-request-tracker", {"event": "request_production", "actor": "ops"}))
    production = asyncio.run(
        transition_handler(
            "team-request-tracker",
            {"event": "approve_production", "actor": "ops", "facts": {"commit_sha": "def456", "release_tag": "tracker-v1"}},
        )
    )

    assert validate["data"]["event_type"] == "app_spec_validated"
    assert sandbox["data"]["registry_snapshot"]["lifecycle_status"] == "sandbox"
    assert request["data"]["approval_request"]["status"] == "pending"
    assert request["data"]["approval_request"]["required_approvers"][1]["kind"] == "data_owner"
    assert production["data"]["registry_snapshot"]["lifecycle_status"] == "production"
    assert production["data"]["registry_snapshot"]["health"]["last_test_run_status"] == "passed"

    snapshots = asyncio.run(snapshots_handler("team-request-tracker"))
    approvals = asyncio.run(approvals_handler("team-request-tracker"))
    evidence = asyncio.run(evidence_handler("request-tracker-prd"))

    assert len(snapshots["data"]) >= 2
    assert approvals["data"][0]["status"] == "approved"
    assert any(event["action"] == "production_deployed" for event in evidence["data"]["events"])


def test_governance_transition_rejects_tier_4_blocklisted_actions() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/admin/projects/{project_id}/governance/transition")]

    result = asyncio.run(
        handler(
            "team-request-tracker",
            {
                "event": "deploy_sandbox",
                "actor": "ops",
                "facts": {"actions": ["price_change"]},
            },
        )
    )

    assert result["data"]["status"] == "rejected"
    assert result["data"]["reason"] == "tier_4_blocklist"
    assert result["data"]["blocked_actions"] == ["price_change"]
