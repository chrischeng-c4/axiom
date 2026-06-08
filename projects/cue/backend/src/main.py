"""Cue Artifact Studio API application."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from mambalibs.http import App, CORSMiddleware, StaticFiles

try:
    from .services import (
        WorkstreamError,
        connector_catalog,
        create_prd_for_workitem,
        diff_app_specs,
        evaluate_policy_and_risk,
        app_spec_preview,
        apply_controlled_app_spec_edit,
        render_tracker_primitives,
        run_approval_runtime_sandbox,
        run_triage_runtime_sandbox,
        run_tracker_workflow_sandbox,
        run_tracker_regression_harness,
        simulate_permissions,
        submit_prompt,
        submit_session_message,
        validate_app_spec,
        workitem_context,
    )
    from .store import store
except ImportError:  # pragma: no cover - supports direct imports in tests and local scripts
    from services import (
        WorkstreamError,
        connector_catalog,
        create_prd_for_workitem,
        diff_app_specs,
        evaluate_policy_and_risk,
        app_spec_preview,
        apply_controlled_app_spec_edit,
        render_tracker_primitives,
        run_approval_runtime_sandbox,
        run_triage_runtime_sandbox,
        run_tracker_workflow_sandbox,
        run_tracker_regression_harness,
        simulate_permissions,
        submit_prompt,
        submit_session_message,
        validate_app_spec,
        workitem_context,
    )
    from store import store

BACKEND_ROOT = Path(__file__).resolve().parents[1]

app = App(
    title="Cue Artifact Studio API",
    description="Prompt-to-governed-app workstream backend",
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
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


def _ok(data: Any) -> dict[str, Any]:
    return {"ok": True, "data": data}


def _error(error: WorkstreamError) -> dict[str, Any]:
    return {"ok": False, **error.payload()}


@app.get("/api/health")
async def api_health() -> dict[str, str]:
    return {
        "status": "healthy",
        "service": "cue-artifact-studio-api",
        "backend": "mamba",
        "frontend": "jet",
    }


@app.get("/health")
async def health() -> dict[str, str]:
    return await api_health()


@app.get("/api/projects")
async def list_projects() -> dict[str, Any]:
    return {"projects": store.list_projects()}


@app.get("/api/projects/{project_id}")
async def get_project(project_id: str) -> dict[str, Any]:
    project = store.get_project(project_id)
    if project is None:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok(project)


@app.get("/api/projects/{project_id}/workitems")
async def list_project_workitems(project_id: str) -> dict[str, Any]:
    if project_id not in store.projects:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok(store.list_workitems(project_id))


@app.get("/api/projects/{project_id}/artifacts")
async def list_project_artifacts(project_id: str) -> dict[str, Any]:
    if project_id not in store.projects:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok(store.list_artifacts(project_id))


@app.get("/api/projects/{project_id}/runtime-tenants")
async def list_project_runtime_tenants(project_id: str) -> dict[str, Any]:
    tenants = store.list_runtime_tenants(project_id)
    if tenants is None:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok(tenants)


@app.get("/api/projects/{project_id}/ownership-namespace")
async def get_project_ownership_namespace(project_id: str) -> dict[str, Any]:
    namespace = store.get_ownership_namespace(project_id)
    if namespace is None:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok(namespace)


@app.post("/api/projects/{project_id}/deployment")
async def post_project_deployment_action(project_id: str, payload: dict[str, Any]) -> dict[str, Any]:
    result = store.deployment_action(
        project_id,
        str(payload.get("action") or ""),
        str(payload.get("actor") or "owner"),
        payload.get("target_ref"),
    )
    if result is None:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok(result)


@app.post("/api/projects/{project_id}/prompts")
async def post_project_prompt(project_id: str, payload: dict[str, Any]) -> dict[str, Any]:
    try:
        return _ok(submit_prompt(project_id, payload))
    except WorkstreamError as error:
        return _error(error)


@app.post("/api/sessions/{session_id}/messages")
async def post_session_message(session_id: str, payload: dict[str, Any]) -> dict[str, Any]:
    try:
        return submit_session_message(session_id, payload)
    except WorkstreamError as error:
        return _error(error)


@app.get("/api/workitems/{workitem_id}/context")
async def get_workitem_context(workitem_id: str) -> dict[str, Any]:
    try:
        return workitem_context(workitem_id)
    except WorkstreamError as error:
        return _error(error)


@app.post("/api/workitems/{workitem_id}/prd")
async def post_workitem_prd(workitem_id: str, payload: dict[str, Any]) -> dict[str, Any]:
    try:
        return _ok(create_prd_for_workitem(workitem_id, payload))
    except WorkstreamError as error:
        return _error(error)


@app.get("/api/admin/workitems")
async def list_admin_workitems() -> dict[str, Any]:
    return _ok(store.list_admin_workitems())


@app.get("/api/admin/hidden-repo-template")
async def get_admin_hidden_repo_template() -> dict[str, Any]:
    return _ok(store.hidden_repo_template_manifest())


@app.get("/api/admin/template-library")
async def get_admin_template_library() -> dict[str, Any]:
    return _ok(store.template_library_manifest())


@app.get("/api/admin/pilot-acceptance-dashboard")
async def get_admin_pilot_acceptance_dashboard() -> dict[str, Any]:
    return _ok(store.pilot_acceptance_dashboard())


@app.get("/api/admin/product-layout")
async def get_admin_product_layout() -> dict[str, Any]:
    return _ok(store.product_layout_manifest())


@app.get("/api/admin/frontend-shell")
async def get_admin_frontend_shell() -> dict[str, Any]:
    return _ok(store.frontend_shell_manifest())


@app.get("/api/admin/artifact-graph")
async def get_admin_artifact_graph() -> dict[str, Any]:
    return _ok(store.artifact_graph_manifest())


@app.get("/api/admin/product-layout/route/{audience}")
async def get_admin_product_layout_route(audience: str) -> dict[str, Any]:
    return _ok(store.resolve_workspace(audience))


@app.post("/api/admin/app-spec/validate")
async def post_admin_app_spec_validate(payload: dict[str, Any]) -> dict[str, Any]:
    try:
        return _ok(validate_app_spec(payload.get("spec", payload), str(payload.get("phase") or "sandbox")))
    except WorkstreamError as error:
        return _error(error)


@app.post("/api/admin/app-spec/diff")
async def post_admin_app_spec_diff(payload: dict[str, Any]) -> dict[str, Any]:
    return _ok(diff_app_specs(payload.get("old", {}), payload.get("new", {})))


@app.post("/api/admin/app-spec/preview")
async def post_admin_app_spec_preview(payload: dict[str, Any]) -> dict[str, Any]:
    return _ok(app_spec_preview(payload.get("spec", payload)))


@app.post("/api/admin/app-spec/controlled-edit")
async def post_admin_app_spec_controlled_edit(payload: dict[str, Any]) -> dict[str, Any]:
    return _ok(apply_controlled_app_spec_edit(payload.get("old", {}), payload.get("edit", {})))


@app.post("/api/admin/tracker/render-primitives")
async def post_admin_tracker_render_primitives(payload: dict[str, Any]) -> dict[str, Any]:
    return _ok(render_tracker_primitives(payload.get("spec", {}), str(payload.get("role") or "viewer")))


@app.get("/api/admin/connectors")
async def get_admin_connector_catalog() -> dict[str, Any]:
    return _ok(connector_catalog())


@app.post("/api/admin/app-spec/permissions/simulate")
async def post_admin_app_spec_permission_simulation(payload: dict[str, Any]) -> dict[str, Any]:
    return _ok(simulate_permissions(payload.get("spec", {}), str(payload.get("role") or "viewer")))


@app.post("/api/admin/app-spec/policy-risk")
async def post_admin_app_spec_policy_risk(payload: dict[str, Any]) -> dict[str, Any]:
    return _ok(evaluate_policy_and_risk(payload.get("spec", payload)))


@app.post("/api/admin/tracker/workflow-sandbox")
async def post_admin_tracker_workflow_sandbox(payload: dict[str, Any]) -> dict[str, Any]:
    return _ok(run_tracker_workflow_sandbox(payload))


@app.post("/api/admin/tracker/regression-harness")
async def post_admin_tracker_regression_harness(payload: dict[str, Any]) -> dict[str, Any]:
    return _ok(run_tracker_regression_harness(payload))


@app.post("/api/admin/triage/runtime-sandbox")
async def post_admin_triage_runtime_sandbox(payload: dict[str, Any]) -> dict[str, Any]:
    return _ok(run_triage_runtime_sandbox(payload))


@app.post("/api/admin/approval/runtime-sandbox")
async def post_admin_approval_runtime_sandbox(payload: dict[str, Any]) -> dict[str, Any]:
    return _ok(run_approval_runtime_sandbox(payload))


@app.get("/api/admin/tracker/test-history")
async def get_admin_tracker_test_history() -> dict[str, Any]:
    return _ok(store.test_history)


@app.get("/api/admin/workitems/{workitem_id}/evidence")
async def get_admin_workitem_evidence(workitem_id: str) -> dict[str, Any]:
    evidence = store.evidence_for_workitem(workitem_id)
    if evidence is None:
        return _error(WorkstreamError("workitem_not_found", f"WorkItem '{workitem_id}' does not exist."))
    return _ok(evidence)


@app.get("/api/admin/projects/{project_id}/registry")
async def get_admin_project_registry(project_id: str) -> dict[str, Any]:
    registry = store.get_registry_entry(project_id)
    if registry is None:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok(registry)


@app.get("/api/admin/projects/{project_id}/goal-review")
async def get_admin_project_goal_review(project_id: str) -> dict[str, Any]:
    review = store.project_goal_review(project_id)
    if review is None:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok(review)


@app.post("/api/admin/projects/{project_id}/goal")
async def post_admin_project_goal(project_id: str, payload: dict[str, Any]) -> dict[str, Any]:
    review = store.update_project_goal(project_id, payload.get("goal", payload), str(payload.get("actor") or "owner"))
    if review is None:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok(review)


@app.post("/api/admin/app-registry/catalog")
async def post_admin_app_registry_catalog(payload: dict[str, Any]) -> dict[str, Any]:
    return _ok(
        store.app_registry_catalog(
            str(payload.get("query") or ""),
            payload.get("filters") or {},
            bool(payload.get("include_internal")),
        )
    )


@app.post("/api/admin/app-registry/duplicates")
async def post_admin_app_registry_duplicates(payload: dict[str, Any]) -> dict[str, Any]:
    return _ok(store.duplicate_app_candidates(str(payload.get("name") or ""), payload.get("owner")))


@app.post("/api/admin/projects/{project_id}/governance/transition")
async def post_admin_project_governance_transition(project_id: str, payload: dict[str, Any]) -> dict[str, Any]:
    result = store.governance_transition(
        project_id,
        str(payload.get("event") or ""),
        str(payload.get("actor") or "cue"),
        payload.get("facts") or {},
    )
    if result is None:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok(result)


@app.get("/api/admin/projects/{project_id}/registry-snapshots")
async def get_admin_project_registry_snapshots(project_id: str) -> dict[str, Any]:
    if project_id not in store.projects:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok([snapshot for snapshot in store.registry_snapshots if snapshot["app_id"] == project_id])


@app.get("/api/admin/projects/{project_id}/approval-requests")
async def get_admin_project_approval_requests(project_id: str) -> dict[str, Any]:
    if project_id not in store.projects:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok([request for request in store.approval_requests if request["app_id"] == project_id])


@app.get("/api/admin/projects/{project_id}/audit")
async def get_admin_project_audit(project_id: str) -> dict[str, Any]:
    events = store.project_audit_events(project_id)
    if events is None:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok(events)


@app.get("/api/admin/projects/{project_id}/metrics")
async def get_admin_project_metrics(project_id: str) -> dict[str, Any]:
    metrics = store.project_metrics(project_id)
    if metrics is None:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok(metrics)


@app.get("/api/admin/projects/{project_id}/health-score")
async def get_admin_project_health_score(project_id: str) -> dict[str, Any]:
    health = store.project_health_score(project_id)
    if health is None:
        return _error(WorkstreamError("project_not_found", f"Project '{project_id}' does not exist."))
    return _ok(health)


static_path = BACKEND_ROOT / "static"
if static_path.exists() and static_path.is_dir():
    app.mount("/", StaticFiles(directory=str(static_path), html=True), name="static")


if __name__ == "__main__":
    import os

    try:
        from .dev_server import run_dev_server
    except ImportError:  # pragma: no cover - supports `python src/main.py`
        from dev_server import run_dev_server

    run_dev_server(
        app,
        host=os.environ.get("CUE_BACKEND_HOST", "127.0.0.1"),
        port=int(os.environ.get("CUE_BACKEND_PORT", "43219")),
    )
