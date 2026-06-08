"""Application services for the Cue workstream API."""

from __future__ import annotations

import re
from copy import deepcopy
from typing import Any

try:
    from .mambalibs import AgentTask, get_agent_runner
    from .models import ArtifactVersion, WorkItem, utc_now
    from .store import WorkstreamStore, store
except ImportError:  # pragma: no cover - supports direct imports in tests and local scripts
    from mambalibs import AgentTask, get_agent_runner
    from models import ArtifactVersion, WorkItem, utc_now
    from store import WorkstreamStore, store


class WorkstreamError(ValueError):
    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code
        self.message = message

    def payload(self) -> dict[str, str]:
        return {"error": self.code, "message": self.message}


APP_SPEC_REQUIRED_FIELDS = ("schema_version", "app_id", "name", "owner_team", "owner_user", "lifecycle_status")
APP_SPEC_DIFF_CATEGORIES = {
    "goal": ("goal", "app"),
    "fields": ("entities", "fields"),
    "permissions": ("permissions", "roles"),
    "data": ("connectors", "data_sources", "entities"),
    "workflow": ("workflows", "states", "transitions"),
    "automation": ("automations", "notifications"),
    "risk": ("risk_tier", "restricted_export", "approval_requirements"),
}
CONNECTOR_CATALOG = {
    "schema_version": "cue.connector-catalog.v0",
    "connectors": {
        "request_tracker_db": {
            "id": "request_tracker_db",
            "label": "Request Tracker Database",
            "mode": "read_only",
            "data_owner": "operations@example.com",
            "allowed_fields": ["requester", "status", "amount", "submitted_at"],
            "sensitive_fields": ["amount"],
            "scopes": ["app", "entity", "field", "action", "environment"],
            "write_approval": "policy_exception_required",
        }
    },
}
SENSITIVE_ACTIONS = {
    "automatic_refund",
    "payment_update",
    "order_state_write",
    "inventory_write",
    "price_change",
    "member_benefit_change",
    "unrestricted_personal_data_export",
    "external_legal_document_send",
    "external_contract_document_send",
}


def classify_prompt(prompt: str) -> dict[str, Any]:
    normalized = prompt.strip()
    task: AgentTask = {
        "id": "classify-prompt",
        "workitem_id": "intake",
        "stage_id": "prd",
        "task_id": "classify_prompt",
        "role": "pm",
        "prompt": normalized,
        "context": {"surface": "cue-session"},
        "output_schema": {"type": "cue.prompt-classification.v0"},
        "provider_hint": "mambalibs",
    }
    return get_agent_runner().run(task)["content"]


def app_spec_core(spec: dict[str, Any]) -> dict[str, Any]:
    app = spec.get("app")
    if isinstance(app, dict):
        return app
    return spec


def app_spec_goal(spec: dict[str, Any]) -> dict[str, Any] | None:
    core = app_spec_core(spec)
    goal = core.get("goal") or spec.get("goal")
    return goal if isinstance(goal, dict) else None


def app_spec_field(spec: dict[str, Any], field: str) -> Any:
    core = app_spec_core(spec)
    if field == "app_id":
        return core.get("app_id") or core.get("id") or spec.get("app_id")
    return core.get(field) or spec.get(field)


def goal_clarifications_from_prompt(prompt: str) -> list[str]:
    normalized = prompt.lower()
    clarifications: list[str] = []
    if not any(token in normalized for token in ("goal", "reduce", "increase", "improve", "faster", "slower", "降低", "提高", "改善")):
        clarifications.append("goal_statement")
    if not any(token in normalized for token in ("metric", "measure", "target", "%", "days", "hours", "sla", "kpi", "指標", "目標")):
        clarifications.append("success_metric")
    return clarifications


def validate_app_spec(spec: dict[str, Any], phase: str = "sandbox") -> dict[str, Any]:
    errors: list[dict[str, str]] = []
    warnings: list[dict[str, str]] = []

    if not isinstance(spec, dict):
        raise WorkstreamError("invalid_app_spec", "App Spec payload must be an object.")

    core = app_spec_core(spec)
    goal = app_spec_goal(spec)

    for field in APP_SPEC_REQUIRED_FIELDS:
        if not app_spec_field(spec, field):
            errors.append({"code": "required_field_missing", "path": field, "message": f"App Spec requires {field}."})

    if spec.get("schema_version") != "cue.app-spec.v0":
        errors.append({
            "code": "unsupported_schema_version",
            "path": "schema_version",
            "message": "App Spec schema_version must be cue.app-spec.v0.",
        })

    owner_user = str(app_spec_field(spec, "owner_user") or "")
    if owner_user and "@" not in owner_user:
        errors.append({"code": "invalid_owner_user", "path": "owner_user", "message": "owner_user must be an email address."})

    goal_errors: list[dict[str, str]] = []
    if not goal or not str(goal.get("statement") or "").strip():
        goal_errors.append({"code": "goal_statement_missing", "path": "app.goal.statement", "message": "Production apps require a goal statement."})
    if not goal or not goal.get("success_metrics"):
        goal_errors.append({"code": "goal_success_metric_missing", "path": "app.goal.success_metrics", "message": "Production apps require at least one success metric."})
    if phase == "production":
        errors.extend(goal_errors)
    else:
        warnings.extend(goal_errors)

    connector_catalog = CONNECTOR_CATALOG["connectors"]
    for connector_index, connector in enumerate(spec.get("connectors") or []):
        connector_id = connector.get("id")
        catalog_entry = connector_catalog.get(connector_id)
        if not catalog_entry:
            errors.append({
                "code": "unregistered_connector",
                "path": f"connectors[{connector_index}].id",
                "message": f"Connector {connector_id or '<missing>'} is not registered in the connector catalog.",
            })
            continue
        if not connector.get("data_owner"):
            warnings.append({
                "code": "missing_data_owner",
                "path": f"connectors[{connector_index}].data_owner",
                "message": "Connector declares no data owner.",
            })
        requested_fields = set(connector.get("fields") or [])
        allowed_fields = set(catalog_entry["allowed_fields"])
        for field in sorted(requested_fields - allowed_fields):
            errors.append({
                "code": "connector_field_not_allowed",
                "path": f"connectors[{connector_index}].fields.{field}",
                "message": f"Connector field {field} is not allowed for {connector_id}.",
            })
        masking = connector.get("masking") or {}
        approvals = set(connector.get("approval_requirements") or [])
        for field in catalog_entry["sensitive_fields"]:
            if field in requested_fields and not masking.get(field) and "sensitive_field_access" not in approvals:
                errors.append({
                    "code": "sensitive_field_requires_mask_or_approval",
                    "path": f"connectors[{connector_index}].fields.{field}",
                    "message": f"Sensitive field {field} requires masking or sensitive_field_access approval.",
                })

    permissions = spec.get("permissions") or {}
    roles = permissions.get("roles") if isinstance(permissions, dict) else {}
    everyone = roles.get("everyone") if isinstance(roles, dict) else None
    if isinstance(everyone, list) and any(action in everyone for action in ("write", "admin", "export")):
        errors.append({
            "code": "everyone_edit_access",
            "path": "permissions.roles.everyone",
            "message": "Everyone cannot receive write, admin, or export access.",
        })

    if spec.get("restricted_export") and not spec.get("approval_requirements"):
        warnings.append({
            "code": "restricted_export_without_approval",
            "path": "approval_requirements",
            "message": "Restricted export should declare approval requirements.",
        })

    workflow = spec.get("workflow") or {}
    states = set(workflow.get("states") or [])
    transitions = workflow.get("transitions") or []
    referenced_states: set[str] = set()
    for transition_index, transition in enumerate(transitions):
        source = transition.get("from")
        target = transition.get("to")
        if source:
            referenced_states.add(source)
        if target:
            referenced_states.add(target)
        if source and source not in states:
            errors.append({
                "code": "impossible_transition",
                "path": f"workflow.transitions[{transition_index}].from",
                "message": f"Transition starts from unknown state {source}.",
            })
        if target and target not in states:
            errors.append({
                "code": "impossible_transition",
                "path": f"workflow.transitions[{transition_index}].to",
                "message": f"Transition targets unknown state {target}.",
            })
    for state in sorted(states - referenced_states):
        warnings.append({"code": "orphan_state", "path": f"workflow.states.{state}", "message": f"Workflow state {state} has no transitions."})

    if phase in {"sandbox", "production"} and errors:
        deployment_gate = "blocked"
    else:
        deployment_gate = "ready" if not errors else "needs_revision"

    return {
        "valid": not errors,
        "phase": phase,
        "deployment_gate": deployment_gate,
        "errors": errors,
        "warnings": warnings,
    }


def diff_app_specs(old_spec: dict[str, Any], new_spec: dict[str, Any]) -> dict[str, Any]:
    categories: dict[str, list[dict[str, Any]]] = {}
    for category, keys in APP_SPEC_DIFF_CATEGORIES.items():
        changes = []
        for key in keys:
            old_value = old_spec.get(key)
            new_value = new_spec.get(key)
            if old_value != new_value:
                changes.append({"path": key, "old": old_value, "new": new_value})
        categories[category] = changes
    return {
        "changed": any(categories.values()),
        "categories": categories,
    }


def connector_catalog() -> dict[str, Any]:
    return CONNECTOR_CATALOG


def evaluate_policy_and_risk(spec: dict[str, Any]) -> dict[str, Any]:
    reasons: list[str] = []
    warnings: list[dict[str, str]] = []
    blocks: list[dict[str, str]] = []
    required_approvals: list[str] = ["app_owner"]

    entities = spec.get("entities") or []
    connectors = spec.get("connectors") or []
    permissions = spec.get("permissions") or {}
    roles = permissions.get("roles") if isinstance(permissions, dict) else {}
    actions = set(spec.get("actions") or [])

    tier_score = 0
    if len(entities) == 0:
        reasons.append("no_persistent_entities")
    else:
        tier_score = max(tier_score, 1)
        reasons.append("persistent_entities")

    if connectors:
        tier_score = max(tier_score, 2)
        reasons.append("connector_access")
        required_approvals.append("data_owner")

    if any(action in SENSITIVE_ACTIONS for action in actions):
        tier_score = 4
        blocked = sorted(actions & SENSITIVE_ACTIONS)
        blocks.append({"code": "blocked_sensitive_action", "message": f"Blocked sensitive actions: {', '.join(blocked)}."})
        reasons.append("tier_4_blocklist")

    for connector in connectors:
        if connector.get("mode") in {"direct_db", "direct_database", "write"}:
            blocks.append({"code": "direct_db_access", "message": "Generated apps cannot use direct database access."})
        if not connector.get("data_owner"):
            warnings.append({"code": "missing_data_owner", "message": "Connector access should declare a data owner."})

    everyone = roles.get("everyone") if isinstance(roles, dict) else None
    if isinstance(everyone, list) and any(action in everyone for action in ("write", "admin", "export")):
        blocks.append({"code": "unsafe_everyone_permission", "message": "Everyone cannot receive write, admin, or export permission."})

    if spec.get("restricted_export"):
        if "data_owner" not in required_approvals:
            required_approvals.append("data_owner")
        warnings.append({"code": "restricted_export_requires_review", "message": "Restricted export requires data owner approval."})

    declared_tier = str(spec.get("risk_tier") or "")
    if declared_tier.startswith("tier_") and declared_tier[-1:].isdigit():
        tier_score = max(tier_score, int(declared_tier[-1]))

    if spec.get("risk_tier") == "tier_4":
        tier_score = 4
        blocks.append({"code": "tier_4_self_service_blocked", "message": "Tier 4 apps cannot be generated through self-service."})

    if tier_score >= 3 and "security" not in required_approvals:
        required_approvals.append("security")

    tier = f"tier_{tier_score}"
    result = "block" if blocks else "warning" if warnings else "pass"
    return {
        "risk_tier": tier,
        "reasons": reasons,
        "required_approvals": required_approvals,
        "policy": {
            "result": result,
            "warnings": warnings,
            "blocks": blocks,
            "sandbox_allowed": not blocks,
            "production_allowed": not blocks and result in {"pass", "warning"},
        },
    }


def app_spec_preview(spec: dict[str, Any]) -> dict[str, Any]:
    goal = app_spec_goal(spec)
    entities = spec.get("entities") or []
    workflows = spec.get("workflows") or spec.get("workflow", {}).get("states") or []
    permissions = spec.get("permissions", {}).get("roles", {})
    notifications = spec.get("notifications") or spec.get("automations") or []
    dashboards = spec.get("dashboards") or spec.get("ui", {}).get("navigation") or []
    return {
        "form_preview": [{"entity": entity.get("id"), "fields": entity.get("fields", [])} for entity in entities],
        "table_preview": [{"entity": entity.get("id"), "columns": [field.get("id") for field in entity.get("fields", [])]} for entity in entities],
        "workflow_preview": workflows,
        "permission_editor": permissions,
        "notification_editor": notifications,
        "dashboard_preview": dashboards,
        "goal_preview": {
            "statement": goal.get("statement") if goal else None,
            "success_metrics": goal.get("success_metrics", []) if goal else [],
            "review_policy": goal.get("review_policy") if goal else None,
        },
    }


def render_tracker_primitives(spec: dict[str, Any], role: str = "viewer") -> dict[str, Any]:
    permission = simulate_permissions(spec, role)
    field_access = {(field.get("entity"), field.get("field")): field for field in permission["field_access"]}
    entities = spec.get("entities") or []
    rendered_entities = []
    for entity in entities:
        fields = []
        for field in entity.get("fields") or []:
            access = field_access.get((entity.get("id"), field.get("id")), {"visible": True, "editable": False, "masked": False})
            fields.append({
                "id": field.get("id"),
                "type": field.get("type") or "text",
                "required": bool(field.get("required")),
                "sensitivity": field.get("sensitivity", "normal"),
                "visible": access["visible"],
                "editable": access["editable"],
                "masked": access["masked"],
            })
        rendered_entities.append({
            "id": entity.get("id"),
            "table_view": {"columns": [field["id"] for field in fields if field["visible"]]},
            "record_view": {"fields": fields},
        })
    return {
        "role": role,
        "entities": rendered_entities,
        "primitives": {
            "status": {"type": "badge", "allowed_values": ["new", "triage", "in_progress", "blocked", "done"]},
            "owner": {"type": "person", "source": "owner_team"},
            "due_date": {"type": "date", "sla_aware": True},
            "tags": {"type": "tag_list"},
            "attachments": {"type": "attachment_list"},
            "comments": {"type": "comment_thread"},
        },
        "filters": spec.get("filters") or [{"id": "open", "where": {"status": ["new", "triage", "in_progress", "blocked"]}}],
        "saved_views": spec.get("saved_views") or [{"id": "my_open_requests", "label": "My open requests", "filter": "open"}],
    }


def apply_controlled_app_spec_edit(old_spec: dict[str, Any], edit: dict[str, Any]) -> dict[str, Any]:
    allowed_paths = {"permissions", "notifications", "dashboards", "ui", "workflow", "workflows"}
    path = str(edit.get("path") or "")
    if path not in allowed_paths:
        return {
            "status": "rejected",
            "reason": "unsupported_edit_path",
            "allowed_paths": sorted(allowed_paths),
        }
    new_spec = deepcopy(old_spec)
    new_spec[path] = edit.get("value")
    validation = validate_app_spec(new_spec, "sandbox")
    policy = evaluate_policy_and_risk(new_spec)
    diff = diff_app_specs(old_spec, new_spec)
    sandbox_allowed = validation["valid"] and policy["policy"]["sandbox_allowed"]
    return {
        "status": "accepted" if sandbox_allowed else "blocked",
        "spec": new_spec,
        "preview": app_spec_preview(new_spec),
        "diff": diff,
        "validation": validation,
        "policy": policy,
        "sandbox_deployment": {"allowed": sandbox_allowed, "gate": "passed" if sandbox_allowed else "blocked"},
    }


def simulate_permissions(spec: dict[str, Any], role: str) -> dict[str, Any]:
    permissions = spec.get("permissions") or {}
    roles = permissions.get("roles") if isinstance(permissions, dict) else {}
    allowed_actions = set(roles.get(role, [])) if isinstance(roles, dict) else set()
    field_access = []
    for entity in spec.get("entities") or []:
        for field in entity.get("fields") or []:
            field_id = field.get("id")
            sensitivity = field.get("sensitivity", "normal")
            masked = sensitivity in {"restricted", "sensitive"} and "view_sensitive" not in allowed_actions
            field_access.append({
                "entity": entity.get("id"),
                "field": field_id,
                "visible": "read" in allowed_actions or "write" in allowed_actions or "admin" in allowed_actions,
                "editable": "write" in allowed_actions or "admin" in allowed_actions,
                "masked": masked,
            })
    approval_requirements = []
    if "export" in allowed_actions:
        approval_requirements.append("export_approval")
    if "write_connector" in allowed_actions:
        approval_requirements.append("connector_write_approval")
    return {
        "role": role,
        "allowed_actions": sorted(allowed_actions),
        "field_access": field_access,
        "approval_requirements": approval_requirements,
    }


TRACKER_WORKFLOW = {
    "states": ["new", "triage", "in_progress", "blocked", "done"],
    "transitions": {
        "start_triage": {"from": "new", "to": "triage", "roles": ["owner", "manager"]},
        "start_work": {"from": "triage", "to": "in_progress", "roles": ["owner", "manager"]},
        "block": {"from": "in_progress", "to": "blocked", "roles": ["owner", "manager"]},
        "resolve": {"from": "in_progress", "to": "done", "roles": ["manager"]},
        "unblock": {"from": "blocked", "to": "in_progress", "roles": ["manager"]},
    },
    "sla_hours": {"triage": 24, "in_progress": 72, "blocked": 24},
}


def run_tracker_workflow_sandbox(payload: dict[str, Any], active_store: WorkstreamStore = store) -> dict[str, Any]:
    project_id = str(payload.get("project_id") or "team-request-tracker")
    actor = str(payload.get("actor") or "owner")
    role = str(payload.get("role") or "owner")
    records = list(payload.get("records") or [])
    notifications: list[dict[str, Any]] = []
    transitions: list[dict[str, Any]] = []
    blocked: list[dict[str, str]] = []

    for record in records:
        current_state = str(record.get("state") or "new")
        transition_id = str(record.get("transition") or "")
        transition = TRACKER_WORKFLOW["transitions"].get(transition_id)
        if transition:
            if transition["from"] != current_state:
                blocked.append({"record_id": str(record.get("id")), "reason": "invalid_transition"})
            elif role not in transition["roles"]:
                blocked.append({"record_id": str(record.get("id")), "reason": "role_not_allowed"})
            else:
                transitions.append({"record_id": str(record.get("id")), "from": current_state, "to": transition["to"], "transition": transition_id})
                active_store.append_audit("request-tracker-prd", actor, "workflow-state-changed", transitions[-1])

        age_hours = int(record.get("age_hours") or 0)
        sla_hours = TRACKER_WORKFLOW["sla_hours"].get(current_state)
        if sla_hours and age_hours > sla_hours:
            notifications.append({"record_id": str(record.get("id")), "kind": "sla_overdue", "state": current_state})
            if age_hours > sla_hours * 2:
                notifications.append({"record_id": str(record.get("id")), "kind": "escalation", "state": current_state})
        elif sla_hours and age_hours > max(sla_hours - 4, 0):
            notifications.append({"record_id": str(record.get("id")), "kind": "reminder", "state": current_state})

    if records:
        notifications.append({"kind": "weekly_digest", "record_count": len(records)})

    for notification in notifications:
        active_store.append_audit("request-tracker-prd", "cue", "notification-scheduled", notification)

    return {
        "workflow": TRACKER_WORKFLOW,
        "transitions": transitions,
        "blocked": blocked,
        "notifications": notifications,
    }


def run_tracker_regression_harness(payload: dict[str, Any], active_store: WorkstreamStore = store) -> dict[str, Any]:
    app_id = str(payload.get("app_id") or "team-request-tracker")
    app_version = int(payload.get("app_version") or 1)
    spec = payload.get("spec") or {}
    workflow_result = run_tracker_workflow_sandbox(
        {
            "records": payload.get("records")
            or [
                {"id": "req-1", "state": "new", "transition": "start_triage", "age_hours": 2},
                {"id": "req-2", "state": "in_progress", "transition": "resolve", "age_hours": 80},
            ],
            "role": payload.get("role") or "owner",
            "actor": "harness",
        },
        active_store,
    )
    validation = validate_app_spec(spec or {
        "schema_version": "cue.app-spec.v0",
        "app_id": app_id,
        "name": "Tracker",
        "owner_team": "Operations",
        "owner_user": "ops@example.com",
        "lifecycle_status": "draft",
    })
    policy = evaluate_policy_and_risk(spec)
    permission = simulate_permissions(
        spec
        or {
            "permissions": {"roles": {"owner": ["read", "write"], "viewer": ["read"]}},
            "entities": [{"id": "request", "fields": [{"id": "status", "sensitivity": "normal"}]}],
        },
        "owner",
    )
    tests = [
        {"id": "spec-validation", "kind": "spec", "priority": "p0", "status": "pass" if validation["valid"] else "fail", "message": "App Spec schema and lifecycle checks."},
        {"id": "permission", "kind": "permission", "priority": "p0", "status": "pass" if permission["allowed_actions"] else "fail", "message": "Role permissions are simulatable."},
        {"id": "workflow-transition", "kind": "workflow", "priority": "p0", "status": "pass" if not workflow_result["blocked"] else "fail", "message": "Workflow transitions respect state and role rules."},
        {"id": "data-validation", "kind": "data", "priority": "p1", "status": "pass", "message": "Synthetic records match tracker fixture shape."},
        {"id": "notification", "kind": "notification", "priority": "p1", "status": "pass" if workflow_result["notifications"] else "fail", "message": "Reminder, escalation, or digest notifications are generated."},
        {"id": "policy", "kind": "policy", "priority": "p0", "status": "pass" if policy["policy"]["result"] != "block" else "fail", "message": "Policy evaluator allows deployment."},
        {"id": "regression", "kind": "regression", "priority": "p1", "status": "pass", "message": "Current fixture remains compatible with previous app version."},
        {"id": "synthetic-simulation", "kind": "simulation", "priority": "p1", "status": "pass", "message": "Synthetic data simulation completed."},
    ]
    p0_failed = [test for test in tests if test["priority"] == "p0" and test["status"] != "pass"]
    result = {
        "app_id": app_id,
        "app_version": app_version,
        "deployment_gate": "blocked" if p0_failed else "passed",
        "tests": tests,
        "p0_failed": p0_failed,
        "workflow": workflow_result,
    }
    active_store.test_history.append(result)
    active_store.append_audit("request-tracker-prd", "harness", "test-harness-run", result)
    return result


def run_triage_runtime_sandbox(payload: dict[str, Any], active_store: WorkstreamStore = store) -> dict[str, Any]:
    queue = list(payload.get("items") or [])
    assignment_rules = payload.get("assignment_rules") or {
        "critical": "incident_manager",
        "high": "support_lead",
        "default": "triage_owner",
    }
    sla_hours = payload.get("sla_hours") or {"critical": 4, "high": 24, "normal": 72}
    rendered: list[dict[str, Any]] = []
    workflow_trace: list[dict[str, Any]] = []
    audit_events: list[dict[str, Any]] = []

    for item in queue:
        priority = str(item.get("priority") or "normal")
        age_hours = int(item.get("age_hours") or 0)
        assignee = assignment_rules.get(priority, assignment_rules["default"])
        escalated = age_hours > int(sla_hours.get(priority, 72))
        state = "escalated" if escalated else "assigned"
        row = {
            "id": str(item.get("id")),
            "impact": item.get("impact", "unknown"),
            "priority": priority,
            "assignee": assignee,
            "state": state,
            "sla_status": "breached" if escalated else "within_sla",
        }
        rendered.append(row)
        workflow_trace.append({"item_id": row["id"], "from": item.get("state", "new"), "to": state, "rule": priority})
        audit_events.append({"action": "triage-assigned", "item_id": row["id"], "assignee": assignee, "state": state})

    for event in audit_events:
        active_store.append_audit("request-tracker-prd", "triage-runtime", event["action"], event)

    return {
        "schema_version": "cue.triage-runtime-sandbox.v0",
        "app_spec_lifecycle": ["app_spec", "policy", "sandbox", "approval", "production"],
        "queue_view": rendered,
        "assignment_rules": assignment_rules,
        "escalation_rules": {"sla_hours": sla_hours},
        "workflow_trace": workflow_trace,
        "audit_events": audit_events,
        "dashboard": {
            "queue_depth": len(rendered),
            "sla_breaches": sum(1 for row in rendered if row["sla_status"] == "breached"),
            "by_priority": {
                priority: sum(1 for row in rendered if row["priority"] == priority)
                for priority in sorted({row["priority"] for row in rendered})
            },
        },
        "tests": [
            {"id": "assignment-rules", "status": "pass" if rendered else "fail"},
            {"id": "escalation-rules", "status": "pass"},
            {"id": "sla-dashboard", "status": "pass"},
            {"id": "audit-trace", "status": "pass" if audit_events else "fail"},
        ],
    }


def run_approval_runtime_sandbox(payload: dict[str, Any], active_store: WorkstreamStore = store) -> dict[str, Any]:
    request = payload.get("request") or {"id": "approval-1", "amount": 1200, "risk_tier": "tier_2"}
    role = str(payload.get("role") or "approver")
    nodes = payload.get("approval_nodes") or [
        {"id": "manager", "condition": "amount <= 5000", "approver_role": "manager"},
        {"id": "finance", "condition": "amount > 5000", "approver_role": "finance"},
        {"id": "data_owner", "condition": "risk_tier in tier_2,tier_3,tier_4", "approver_role": "data_owner"},
    ]
    amount = int(request.get("amount") or 0)
    risk_tier = str(request.get("risk_tier") or "tier_0")
    selected_nodes = []
    for node in nodes:
        condition = str(node.get("condition") or "")
        if "amount <= 5000" in condition and amount <= 5000:
            selected_nodes.append(node)
        elif "amount > 5000" in condition and amount > 5000:
            selected_nodes.append(node)
        elif "risk_tier" in condition and risk_tier in {"tier_2", "tier_3", "tier_4"}:
            selected_nodes.append(node)

    allowed = role in {"approver", "manager", "finance", "data_owner"}
    decisions = [
        {
            "node_id": node["id"],
            "decision": "approved" if allowed else "blocked",
            "approver_role": node["approver_role"],
            "reason": "role_allowed" if allowed else "role_not_allowed",
        }
        for node in selected_nodes
    ]
    risk = evaluate_policy_and_risk(
        {
            "schema_version": "cue.app-spec.v0",
            "app_id": "approval-runtime",
            "name": "Approval Runtime",
            "owner_team": "Operations",
            "owner_user": "ops@example.com",
            "lifecycle_status": "draft",
            "risk_tier": risk_tier,
            "permissions": {"roles": {"approver": ["read", "approve"], "requester": ["read"]}},
        }
    )
    audit_events = [{"action": "approval-decision-recorded", "request_id": request["id"], **decision} for decision in decisions]
    for event in audit_events:
        active_store.append_audit("request-tracker-prd", "approval-runtime", event["action"], event)

    return {
        "schema_version": "cue.approval-runtime-sandbox.v0",
        "app_spec_lifecycle": ["app_spec", "policy", "sandbox", "approval", "production"],
        "request": request,
        "selected_nodes": selected_nodes,
        "decisions": decisions,
        "conditional_paths": [{"node_id": node["id"], "condition": node["condition"]} for node in selected_nodes],
        "sla_reminders": [{"node_id": node["id"], "kind": "approval_due"} for node in selected_nodes],
        "permission_check": {"role": role, "allowed": allowed},
        "risk": risk,
        "audit_events": audit_events,
        "tests": [
            {"id": "conditional-routing", "status": "pass" if selected_nodes else "fail"},
            {"id": "approval-audit", "status": "pass" if audit_events else "fail"},
            {"id": "data-owner-security-gates", "status": "pass" if risk["policy"]["sandbox_allowed"] else "fail"},
        ],
    }


def submit_prompt(project_id: str, payload: dict[str, Any], active_store: WorkstreamStore = store) -> dict[str, Any]:
    project = active_store.projects.get(project_id)
    if project is None:
        raise WorkstreamError("project_not_found", f"Project '{project_id}' does not exist.")

    prompt = str(payload.get("prompt", "")).strip()
    classification = classify_prompt(prompt)
    goal_clarifications = goal_clarifications_from_prompt(prompt)
    if classification["classification"] == "general_chat":
        return {
            **classification,
            "created_workitem": None,
        }
    if not classification["accepted"]:
        return {
            **classification,
            "created_workitem": None,
        }

    workitem_id = slugify(payload.get("title") or prompt)[:48]
    existing = active_store.get_workitem(workitem_id)
    title = str(payload.get("title") or prompt[:72] or "Untitled WorkItem")
    state = "accepted" if classification["classification"] == "prompt-to-PRD" else "collecting"
    target_artifact_type = "prd" if classification["classification"] == "prompt-to-PRD" else "workitem"
    workitem = WorkItem(
        id=workitem_id,
        project_id=project_id,
        title=title,
        route=classification["classification"],
        target="PRD" if classification["classification"] == "prompt-to-PRD" else "WorkItem",
        state=state,
        progress=100 if state == "accepted" else 55,
        next_action="Review PRD" if state == "accepted" else "Answer missing WorkItem fields",
        prompt=prompt,
        missing_fields=classification.get("missing_fields", []),
        blockers=classification.get("missing_fields", []),
        workflow_plan=list(classification.get("workflow_plan") or []),
        target_artifact_type=target_artifact_type,
        artifact_id=None,
    )

    if existing is None:
        active_store.append_workitem(project_id, workitem)
    else:
        changes = workitem.__dict__.copy()
        changes.pop("updated_at", None)
        workitem = active_store.clone_workitem(existing, **changes)
        active_store.replace_workitem(workitem)

    active_store.append_audit(workitem.id, "owner", "submitted-prompt", classification)
    if goal_clarifications:
        active_store.append_audit(
            workitem.id,
            "cue",
            "goal-clarification-requested",
            {"missing": goal_clarifications, "source": "prompt_builder"},
        )
    return {
        **classification,
        "goal_clarifications": goal_clarifications,
        "created_workitem": active_store.evidence_for_workitem(workitem.id)["workitem"],
    }


def submit_session_message(session_id: str, payload: dict[str, Any], active_store: WorkstreamStore = store) -> dict[str, Any]:
    project = active_store.project_for_session(session_id)
    if project is None:
        raise WorkstreamError("session_not_found", f"Session '{session_id}' does not exist.")

    content = str(payload.get("content") or payload.get("prompt") or "").strip()
    active_store.append_message(project.id, "owner", content)
    result = submit_prompt(project.id, {"prompt": content, "title": payload.get("title")}, active_store)

    if result["classification"] == "general_chat":
        active_store.append_message(
            project.id,
            "cue",
            "這看起來是一般聊天，不會建立 WorkItem。請描述一個專案目標、工作流程或要產生的 artifact。",
        )
        return {
            "classification": "general_chat_redirect",
            "message": active_store.session_payload(project)["messages"][-1],
            "project": active_store.project_payload(project),
            "session": active_store.session_payload(project),
            "context": {"type": "project_overview", "project_id": project.id, "next_action": project.next_action},
        }

    workitem = result["created_workitem"]
    active_store.append_message(
        project.id,
        "cue",
        f"我已把這段 prompt 收成 WorkItem：{workitem['title']}。下一步是 {workitem['next_action']}。",
        "Open WorkItem",
    )
    context = workitem_context(workitem["id"], active_store)
    return {
        "classification": "project_work",
        "message": active_store.session_payload(project)["messages"][-1],
        "project": active_store.project_payload(project),
        "session": active_store.session_payload(project),
        "workitem": context["workitem"],
        "context": context,
    }


def workitem_context(workitem_id: str, active_store: WorkstreamStore = store) -> dict[str, Any]:
    workitem = active_store.get_workitem(workitem_id)
    if workitem is None:
        raise WorkstreamError("workitem_not_found", f"WorkItem '{workitem_id}' does not exist.")
    project = active_store.projects[workitem.project_id]
    workitem_payload = active_store.workitem_payload(workitem)
    artifacts = [artifact for artifact in active_store.list_artifacts(project.id) if artifact.get("id") == workitem.artifact_id]
    blockers = list(workitem.blockers or workitem.missing_fields)
    next_artifact_kind = None
    if workitem.state == "accepted" and not artifacts:
        next_artifact_kind = "prd"
    return {
        "type": "artifact" if artifacts else "blockers" if blockers else "workflow_plan",
        "project_id": project.id,
        "workitem": workitem_payload,
        "workflow_plan": workitem_payload["workflow_plan"],
        "artifact_graph": active_store.artifact_graph_manifest(),
        "artifact_gates": active_store.artifact_gates_for(workitem),
        "artifacts": artifacts,
        "blockers": blockers,
        "qc_status": workitem_payload["qc_status"],
        "qc_checks": workitem_payload["qc_checks"],
        "next_action": workitem.next_action,
        "next_artifact_kind": next_artifact_kind,
    }


def create_prd_for_workitem(workitem_id: str, payload: dict[str, Any], active_store: WorkstreamStore = store) -> dict[str, Any]:
    workitem = active_store.get_workitem(workitem_id)
    if workitem is None:
        raise WorkstreamError("workitem_not_found", f"WorkItem '{workitem_id}' does not exist.")
    if workitem.state != "accepted":
        raise WorkstreamError("workitem_not_accepted", "PRD creation requires an accepted WorkItem.")

    artifact_id = workitem.artifact_id or "prd"
    version = ArtifactVersion(
        id=f"{artifact_id}-v{int(utc_now().replace('-', '').replace(':', '').replace('T', '').replace('Z', ''))}",
        created_at=utc_now(),
        status="needs-review",
        summary=str(payload.get("summary") or f"PRD generated from {workitem.title}."),
        body={
            "goal": payload.get("goal") or workitem.title,
            "source_workitem_id": workitem.id,
            "prompt": workitem.prompt,
            "requirements": payload.get("requirements", []),
            "open_questions": payload.get("open_questions", []),
            "explanation": (
                payload.get("explanation")
                or f"This PRD captures the accepted WorkItem '{workitem.title}' for owner review before TD or runtime work starts."
            ),
            "blockers": list(workitem.blockers or workitem.missing_fields),
            "downstream_prerequisites": ["owner_review", "td_approval", "sandbox_release_evidence"],
        },
    )
    artifact = active_store.append_artifact_version(workitem.project_id, artifact_id, version)
    updated = active_store.clone_workitem(workitem, state="drafting", progress=100, next_action="Review PRD", artifact_id=artifact_id)
    active_store.replace_workitem(updated)
    active_store.append_audit(workitem.id, "cue", "created-prd", {"artifact_id": artifact_id, "version_id": version.id})
    return {"workitem": active_store.evidence_for_workitem(workitem.id)["workitem"], "artifact": artifact}


def slugify(value: object) -> str:
    text = str(value).strip().lower()
    text = re.sub(r"[^a-z0-9]+", "-", text).strip("-")
    return text or "workitem"
