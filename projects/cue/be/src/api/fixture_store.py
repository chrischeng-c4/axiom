"""Fixture-backed Cue workstream state.

This module is intentionally plain Python data so the first Cue API slice stays
portable across the current Mamba target and CPython test stubs.
"""

import copy
from datetime import datetime, timezone

try:
    from .generation import get_generator, slug
except ImportError:
    from generation import get_generator, slug


def _now():
    return datetime.now(timezone.utc).isoformat()


def _message(message_id, speaker, body, action=None):
    payload = {
        "id": message_id,
        "speaker": speaker,
        "body": body,
        "created_at": _now(),
    }
    if action:
        payload["action"] = action
    return payload


def _qc(status, *checks):
    return {
        "qc_status": status,
        "qc_checks": [
            {"id": check[0], "label": check[1], "status": check[2], "summary": check[3]}
            for check in checks
        ],
    }


PROJECTS = {
    "team-request-tracker": {
        "id": "team-request-tracker",
        "name": "Team Request Tracker",
        "owner": "Operations",
        "status": "needs-review",
        "next_action": "Review PRD",
        "summary": "WorkItem accepted; PRD is waiting for owner review.",
        "active_session_id": "session-request-tracker",
        "sessions": [
            {
                "id": "session-request-tracker",
                "project_id": "team-request-tracker",
                "title": "Request tracker intake",
                "messages": [
                    _message(
                        "m1",
                        "owner",
                        "我們需要一個內部請求追蹤流程，讓同事送單，Operations 可以分派和追狀態。",
                    ),
                    _message(
                        "m2",
                        "cue",
                        "我先把這個 prompt 收成 WorkItem，判斷路由是 prompt-to-PRD。WorkItem 已建立；接著整理 PRD，請確認目標和使用者。",
                        "Open PRD",
                    ),
                    _message("m3", "cue", "目前還需要確認 production owner 和資料保存期限。PRD 通過後 TD 才能開始。"),
                ],
            }
        ],
        "stages": [
            {"id": "workitem", "label": "WorkItem", "state": "done", "detail": "Prompt accepted as prompt-to-PRD."},
            {"id": "prd", "label": "PRD", "state": "in-progress", "detail": "Goal, users, fields, and success metric drafted."},
            {"id": "td", "label": "TD", "state": "not-started", "detail": "Starts after PRD approval."},
            {"id": "website", "label": "Website", "state": "not-started", "detail": "Runtime artifact is locked by TD."},
        ],
        "workitems": [
            {
                "id": "request-tracker-prd",
                "project_id": "team-request-tracker",
                "title": "Create request tracker PRD",
                "route": "prompt-to-PRD",
                "target": "PRD",
                "state": "accepted",
                "progress": 100,
                "next_action": "Review PRD",
                "blockers": [],
                "workflow_plan": [
                    {"id": "prd", "label": "PRD", "state": "in-progress", "depends_on": []},
                    {"id": "td", "label": "TD", "state": "not-started", "depends_on": ["prd"]},
                    {"id": "website", "label": "Website", "state": "not-started", "depends_on": ["td"]},
                ],
                **_qc(
                    "pass",
                    ("intent", "Intent is project work", "pass", "Prompt maps to a governed PRD artifact route."),
                    ("minimum_scope", "Minimum scope captured", "pass", "Owner goal, users, and workflow target are present."),
                ),
            },
            {
                "id": "request-tracker-retention",
                "project_id": "team-request-tracker",
                "title": "Confirm data retention",
                "route": "prompt-to-PRD",
                "target": "PRD",
                "state": "collecting",
                "progress": 62,
                "next_action": "Answer retention question",
                "blockers": ["Confirm production owner", "Confirm data retention window"],
                "workflow_plan": [
                    {"id": "prd", "label": "PRD", "state": "blocked", "depends_on": []},
                    {"id": "td", "label": "TD", "state": "not-started", "depends_on": ["prd"]},
                    {"id": "website", "label": "Website", "state": "not-started", "depends_on": ["td"]},
                ],
                **_qc(
                    "needs_input",
                    ("owner", "Production owner", "needs_input", "Owner must confirm production ownership."),
                    ("retention", "Data retention", "needs_input", "Retention window is required before PRD generation."),
                ),
            },
        ],
        "artifacts": [
            {
                "id": "request-tracker-prd-v1",
                "workitem_id": "request-tracker-prd",
                "label": "Project PRD",
                "kind": "prd",
                "status": "Needs review",
                "summary": "Owner-facing request tracker PRD draft.",
                **_qc(
                    "pending",
                    ("prd_review", "PRD owner review", "pending", "Owner review must complete before TD starts."),
                ),
                "versions": [{"id": "request-tracker-prd-v1", "version": 1, "status": "current"}],
            }
        ],
    },
    "weekly-ops-report": {
        "id": "weekly-ops-report",
        "name": "Weekly Ops Report",
        "owner": "Revenue Ops",
        "status": "in-progress",
        "next_action": "Complete WorkItem",
        "summary": "WorkItem is collecting the minimum details before PRD generation.",
        "active_session_id": "session-weekly-ops-report",
        "sessions": [
            {
                "id": "session-weekly-ops-report",
                "project_id": "weekly-ops-report",
                "title": "Weekly report intake",
                "messages": [
                    _message("m1", "owner", "每週幫我整理 pipeline 和逾期 account，寄給主管。"),
                    _message("m2", "cue", "我先建立 WorkItem，但還缺收件人、資料來源和寄送時間。確認後才會產生 PRD artifact。"),
                ],
            }
        ],
        "stages": [
            {"id": "workitem", "label": "WorkItem", "state": "in-progress", "detail": "Missing owner, recipients, and data source."},
            {"id": "prd", "label": "PRD", "state": "not-started", "detail": "Starts after WorkItem acceptance."},
            {"id": "td", "label": "TD", "state": "not-started", "detail": "Waiting on PRD approval."},
            {"id": "website", "label": "Website", "state": "not-started", "detail": "Report artifact is locked by TD."},
        ],
        "workitems": [
            {
                "id": "weekly-report-intake",
                "project_id": "weekly-ops-report",
                "title": "Collect report basics",
                "route": "prompt-to-WorkItem",
                "target": "WorkItem",
                "state": "collecting",
                "progress": 48,
                "next_action": "Add recipients and data source",
                "blockers": ["Add recipients", "Select data source", "Confirm send time"],
                "workflow_plan": [
                    {"id": "prd", "label": "PRD", "state": "blocked", "depends_on": []},
                    {"id": "td", "label": "TD", "state": "not-started", "depends_on": ["prd"]},
                    {"id": "website", "label": "Website", "state": "not-started", "depends_on": ["td"]},
                ],
                **_qc(
                    "needs_input",
                    ("recipients", "Recipients", "needs_input", "Report recipients are missing."),
                    ("data_source", "Data source", "needs_input", "Source system must be selected."),
                ),
            }
        ],
        "artifacts": [],
    },
}


def _project_for_session(session_id):
    for project in PROJECTS.values():
        for session in project["sessions"]:
            if session["id"] == session_id:
                return project
    return None


def _project_for_workitem(workitem_id):
    for project in PROJECTS.values():
        for workitem in project["workitems"]:
            if workitem["id"] == workitem_id:
                return project
    return None


def _find_workitem(project, workitem_id):
    for workitem in project["workitems"]:
        if workitem["id"] == workitem_id:
            return workitem
    return None


def list_projects():
    return {"projects": [copy.deepcopy(project) for project in PROJECTS.values()]}


def get_project(project_id):
    project = PROJECTS.get(project_id)
    if not project:
        return {"error": {"code": "not_found", "message": "Project not found"}}
    return copy.deepcopy(project)


def list_sessions(project_id):
    project = PROJECTS.get(project_id)
    if not project:
        return {"error": {"code": "not_found", "message": "Project not found"}}
    return {"sessions": copy.deepcopy(project["sessions"])}


def create_session(project_id, payload):
    project = PROJECTS.get(project_id)
    if not project:
        return {"error": {"code": "not_found", "message": "Project not found"}}
    session_id = f"session-{project_id}-{len(project['sessions']) + 1}"
    session = {
        "id": session_id,
        "project_id": project_id,
        "title": payload.get("title") or "New project workstream",
        "messages": [],
    }
    project["sessions"].append(session)
    project["active_session_id"] = session_id
    return {"session": copy.deepcopy(session), "project": copy.deepcopy(project)}


def get_session(session_id):
    project = _project_for_session(session_id)
    if not project:
        return {"error": {"code": "not_found", "message": "Session not found"}}
    for session in project["sessions"]:
        if session["id"] == session_id:
            return {"session": copy.deepcopy(session), "project": copy.deepcopy(project)}
    return {"error": {"code": "not_found", "message": "Session not found"}}


def list_workitems(project_id):
    project = PROJECTS.get(project_id)
    if not project:
        return {"error": {"code": "not_found", "message": "Project not found"}}
    return {"workitems": copy.deepcopy(project["workitems"])}


def list_artifacts(project_id):
    project = PROJECTS.get(project_id)
    if not project:
        return {"error": {"code": "not_found", "message": "Project not found"}}
    return {"artifacts": copy.deepcopy(project["artifacts"])}


def workitem_context(workitem_id):
    project = _project_for_workitem(workitem_id)
    if not project:
        return {"error": {"code": "not_found", "message": "WorkItem not found"}}
    workitem = _find_workitem(project, workitem_id)
    artifacts = [artifact for artifact in project["artifacts"] if artifact.get("workitem_id") == workitem_id]
    next_artifact = _next_artifact_kind(workitem, artifacts)
    context_type = "blockers" if workitem["blockers"] else "workflow_plan"
    if artifacts:
        context_type = "artifact"
    return {
        "type": context_type,
        "project_id": project["id"],
        "workitem": copy.deepcopy(workitem),
        "workflow_plan": copy.deepcopy(workitem["workflow_plan"]),
        "artifacts": copy.deepcopy(artifacts),
        "blockers": copy.deepcopy(workitem["blockers"]),
        "qc_status": workitem["qc_status"],
        "qc_checks": copy.deepcopy(workitem["qc_checks"]),
        "next_action": workitem["next_action"],
        "next_artifact_kind": next_artifact,
    }


def post_message(session_id, payload):
    project = _project_for_session(session_id)
    if not project:
        return {"error": {"code": "not_found", "message": "Session not found"}}
    session = None
    for candidate in project["sessions"]:
        if candidate["id"] == session_id:
            session = candidate
            break
    if session is None:
        return {"error": {"code": "not_found", "message": "Session not found"}}

    content = (payload or {}).get("content", "").strip()
    if not content:
        return {"error": {"code": "invalid_message", "message": "Message content is required"}}

    session["messages"].append(_message(f"m{len(session['messages']) + 1}", "owner", content))

    decision = get_generator().classify_prompt(content)
    if decision["classification"] == "general_chat_redirect":
        cue_message = _message(
            f"m{len(session['messages']) + 1}",
            "cue",
            decision["reply"],
        )
        session["messages"].append(cue_message)
        return {
            "classification": "general_chat_redirect",
            "message": copy.deepcopy(cue_message),
            "project": copy.deepcopy(project),
            "session": copy.deepcopy(session),
            "context": {"type": "project_overview", "project_id": project["id"], "next_action": project["next_action"]},
        }

    workitem = _ensure_workitem_for_prompt(project, decision)
    cue_message = _message(
        f"m{len(session['messages']) + 1}",
        "cue",
        f"我已把這段 prompt 收成 WorkItem：{workitem['title']}。目前 workflow 是 PRD -> TD -> Website，下一步是 {workitem['next_action']}。",
        "Open WorkItem",
    )
    session["messages"].append(cue_message)
    project["active_session_id"] = session_id
    return {
        "classification": "project_work",
        "message": copy.deepcopy(cue_message),
        "project": copy.deepcopy(project),
        "session": copy.deepcopy(session),
        "workitem": copy.deepcopy(workitem),
        "context": workitem_context(workitem["id"]),
    }


def run_artifact(workitem_id, payload):
    project = _project_for_workitem(workitem_id)
    if not project:
        return {"error": {"code": "not_found", "message": "WorkItem not found"}}
    workitem = _find_workitem(project, workitem_id)
    if workitem["state"] not in ["accepted", "done"]:
        return {
            "status": "rejected",
            "reason": "workitem_not_accepted",
            "message": "WorkItem must be accepted before creating artifacts.",
            "project": copy.deepcopy(project),
            "context": workitem_context(workitem_id),
        }

    requested_kind = (payload or {}).get("kind") or _next_artifact_kind(workitem, project["artifacts"]) or "prd"
    artifact_id = f"{workitem_id}-{requested_kind}-v1"
    existing = [artifact for artifact in project["artifacts"] if artifact["id"] == artifact_id]
    generated = get_generator().create_artifact(workitem, requested_kind)
    if not existing:
        project["artifacts"].append(generated["artifact"])

    _mark_stage(project, requested_kind, "in-progress", f"{requested_kind.upper()} artifact is ready for owner review.")
    workitem["progress"] = max(workitem["progress"], 100 if requested_kind == "website" else 76)
    workitem["next_action"] = "Review artifact"
    return {
        "status": "created",
        "qc_result": generated["qc_result"],
        "project": copy.deepcopy(project),
        "context": workitem_context(workitem_id),
    }


def _ensure_workitem_for_prompt(project, decision):
    title = decision["title"]
    workitem_id = f"{project['id']}-{slug(title)}"
    existing = _find_workitem(project, workitem_id)
    if existing:
        existing["next_action"] = "Create PRD artifact"
        existing["state"] = "accepted"
        existing["progress"] = max(existing["progress"], 55)
        return existing

    workitem = {
        "id": workitem_id,
        "project_id": project["id"],
        "title": title,
        "route": decision["route"],
        "target": decision["target"],
        "state": "accepted",
        "progress": 55,
        "next_action": "Create PRD artifact",
        "blockers": [],
        "workflow_plan": decision["workflow_plan"],
        "qc_status": decision["qc_status"],
        "qc_checks": decision["qc_checks"],
    }
    project["workitems"].append(workitem)
    project["summary"] = "WorkItem accepted; PRD -> TD -> Website workflow is ready."
    project["next_action"] = "Create PRD artifact"
    project["status"] = "in-progress"
    project["stages"] = [
        {"id": "workitem", "label": "WorkItem", "state": "done", "detail": "Prompt accepted and converted to workflow state."},
        {"id": "prd", "label": "PRD", "state": "in-progress", "detail": "Ready to draft from WorkItem."},
        {"id": "td", "label": "TD", "state": "not-started", "detail": "Starts after PRD review."},
        {"id": "website", "label": "Website", "state": "not-started", "detail": "Generated after TD approval."},
    ]
    return workitem


def _next_artifact_kind(workitem, artifacts):
    kinds = {artifact.get("kind") for artifact in artifacts}
    if "prd" not in kinds:
        return "prd"
    if "td" not in kinds:
        return "td"
    if workitem.get("target") == "Website" and "website" not in kinds:
        return "website"
    return None


def _mark_stage(project, stage_id, state, detail):
    for stage in project["stages"]:
        if stage["id"] == stage_id:
            stage["state"] = state
            stage["detail"] = detail
            return
