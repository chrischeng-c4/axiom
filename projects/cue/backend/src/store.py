"""Fixture-backed store for the first Cue Artifact Studio backend slice."""

from __future__ import annotations

import json
from copy import deepcopy
from dataclasses import replace
from pathlib import Path
from typing import Any, Literal
from uuid import uuid4

try:
    from .mambalibs import PgKit, RecordingPgKit, pg_insert, pg_update
    from .models import (
        AdminReviewTicket,
        Artifact,
        ArtifactRepository,
        ArtifactVersion,
        AuditEvent,
        Message,
        Project,
        Stage,
        WorkItem,
        to_payload,
        utc_now,
    )
except ImportError:  # pragma: no cover - supports direct `python src/main.py` style imports
    from mambalibs import PgKit, RecordingPgKit, pg_insert, pg_update
    from models import (
        AdminReviewTicket,
        Artifact,
        ArtifactRepository,
        ArtifactVersion,
        AuditEvent,
        Message,
        Project,
        Stage,
        WorkItem,
        to_payload,
        utc_now,
    )


WORKFLOW_NODE_AGENTS: dict[str, dict[str, str]] = {
    "prd": {
        "agent_role": "pm",
        "agent_label": "PM agent",
        "agent_task": "Shape the owner goal, users, constraints, and success criteria.",
    },
    "td": {
        "agent_role": "architect",
        "agent_label": "Architect agent",
        "agent_task": "Turn the approved PRD into system contracts and delivery design.",
    },
    "website": {
        "agent_role": "designer",
        "agent_label": "Designer agent",
        "agent_task": "Produce the owner-facing website artifact after TD approval.",
    },
    "codebase": {
        "agent_role": "dev",
        "agent_label": "Dev agent",
        "agent_task": "Implement source, schemas, delivery assets, and generated artifacts.",
    },
    "test": {
        "agent_role": "qa_policy",
        "agent_label": "QA/policy agent",
        "agent_task": "Run quality, policy, and workflow acceptance checks.",
    },
    "deployment": {
        "agent_role": "release",
        "agent_label": "Release agent",
        "agent_task": "Prepare sandbox release evidence and promotion gates.",
    },
    "operation": {
        "agent_role": "data",
        "agent_label": "Data agent",
        "agent_task": "Maintain runtime metrics and operations dashboard evidence.",
    },
}

HIDDEN_REPO_TEMPLATE_FILES = [
    "app-spec.json",
    "policy.json",
    "permissions.json",
    "connectors.json",
    ".gitlab-ci.yml",
    "README.md",
    "tests/permission-tests.json",
    "tests/workflow-tests.json",
    "tests/policy-tests.json",
    "generated/runtime-config.json",
    "generated/ui-manifest.json",
    "releases/",
]

TIER_4_BLOCKLIST = {
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

CUE_ROOT = Path(__file__).resolve().parents[2]


def workflow_step(id: str, label: str, state: str, depends_on: list[str]) -> dict[str, Any]:
    return {
        "id": id,
        "label": label,
        "state": state,
        "depends_on": depends_on,
        **WORKFLOW_NODE_AGENTS.get(id, {}),
    }


def project_stage(id: str, label: str, state: str, detail: str) -> Stage:
    agent = WORKFLOW_NODE_AGENTS.get(id, {})
    return Stage(
        id,
        label,
        state,  # type: ignore[arg-type]
        detail,
        agent.get("agent_role"),  # type: ignore[arg-type]
        agent.get("agent_label"),
        agent.get("agent_task"),
    )


def _seed_projects() -> dict[str, Project]:
    prd_version = ArtifactVersion(
        id="prd-v1",
        created_at="2026-05-12T10:30:00Z",
        status="needs-review",
        summary="Initial PRD for the internal request tracker.",
        body={
            "goal": "Create a governed internal request tracker for Operations.",
            "users": ["requester", "operations-owner"],
            "success_metric": "Every request has an owner, state, and next action.",
            "open_questions": ["production owner", "data retention period"],
        },
    )
    project = Project(
        id="team-request-tracker",
        name="Team Request Tracker",
        owner="Operations",
        owner_namespace="team",
        risk_tier="tier_2",
        lifecycle_status="active",
        current_workstream_id="request-tracker-prd",
        status="needs-review",
        next_action="Review PRD",
        summary="WorkItem accepted; PRD is waiting for owner review.",
        stages=[
            Stage("workitem", "WorkItem", "done", "Prompt accepted as prompt-to-PRD."),
            project_stage("prd", "PRD", "in-progress", "Goal, users, fields, and success metric drafted."),
            project_stage("td", "TD", "not-started", "Starts after PRD approval."),
            project_stage("website", "Website", "not-started", "Interactive workflow artifact is locked by TD."),
        ],
        messages=[
            Message(
                "m1",
                "owner",
                "我們需要一個內部請求追蹤流程，讓同事送單，Operations 可以分派和追狀態。",
            ),
            Message(
                "m2",
                "cue",
                "我先把這個 prompt 收成 WorkItem，判斷路由是 prompt-to-PRD。WorkItem 已建立；接著整理 PRD，請確認目標和使用者。",
                "Open PRD",
            ),
            Message("m3", "cue", "目前還需要確認 production owner 和資料保存期限。PRD 通過後 TD 才能開始。"),
        ],
        artifacts=[Artifact("prd", "Project PRD", "prd", "Needs review", [prd_version])],
        workitems=[
            WorkItem(
                id="request-tracker-prd",
                project_id="team-request-tracker",
                title="Create request tracker PRD",
                route="prompt-to-PRD",
                target="PRD",
                state="accepted",
                progress=100,
                next_action="Review PRD",
                prompt="Create an internal request tracker for Operations.",
                target_artifact_type="prd",
                artifact_id="prd",
            ),
            WorkItem(
                id="request-tracker-retention",
                project_id="team-request-tracker",
                title="Confirm data retention",
                route="prompt-to-PRD",
                target="PRD",
                state="collecting",
                progress=62,
                next_action="Answer retention question",
                prompt="How long should request data be retained?",
                missing_fields=["retention_period"],
                blockers=["retention_period"],
                target_artifact_type="prd",
            ),
        ],
    )
    return {project.id: project}


class WorkstreamStore:
    def __init__(self, pgkit: PgKit | None = None) -> None:
        self.pgkit = pgkit or RecordingPgKit({"schema": "cue"})
        self.projects = _seed_projects()
        self.repositories: dict[str, ArtifactRepository] = {
            "team-request-tracker": ArtifactRepository(
                id="repo-team-request-tracker",
                project_id="team-request-tracker",
                backend="fixture",
                status="fixture_backed",
                gitlab_full_path="cue-generated-apps/teams/operations/t-operations-team-request-tracker",
                template_version="cue-hidden-repo-v0",
                hidden_gitlab_project_id=10001,
                current_spec_ref="8a58d2e59c7f409184b85d9b46a0f45e3f28de8f",
                sandbox_ref="8a58d2e59c7f409184b85d9b46a0f45e3f28de8f",
                production_release_tag=None,
            )
        }
        self.ownership_namespaces: dict[str, dict[str, Any]] = {
            "team-request-tracker": {
                "schema_version": "cue.ownership-namespace.v0",
                "app_id": "t-operations-team-request-tracker",
                "namespace": "team",
                "display_name": "Team Request Tracker",
                "owner": {
                    "owner_user": "ops-lead@example.com",
                    "owner_team": "Operations",
                    "data_owner": None,
                },
                "platform_owner": "cue-platform@example.com",
                "emergency_contact": "cue-oncall@example.com",
                "quota_policy": "team_default",
                "transfer_policy": "manager_required",
                "visibility": "team_visible",
                "orphan_state": "healthy",
                "role_assignments": [
                    {
                        "role": "creator",
                        "principal_type": "user",
                        "principal": "ops-lead@example.com",
                        "source": "direct",
                        "expires_at": None,
                    },
                    {
                        "role": "app_owner",
                        "principal_type": "user",
                        "principal": "ops-lead@example.com",
                        "source": "direct",
                        "expires_at": None,
                    },
                    {
                        "role": "viewer",
                        "principal_type": "team",
                        "principal": "Operations",
                        "source": "iam",
                        "expires_at": None,
                    },
                ],
                "quota_state": {
                    "sandbox_apps_count": 3,
                    "production_apps_count": 1,
                    "active_exceptions": [],
                },
                "gitlab_mapping": {
                    "root_group": "cue-generated-apps",
                    "group_path": "teams/operations",
                    "project_path": "t-operations-team-request-tracker",
                    "full_path": "cue-generated-apps/teams/operations/t-operations-team-request-tracker",
                    "user_visible": False,
                },
                "audit": {
                    "created_by": "ops-lead@example.com",
                    "created_at": "2026-05-08T00:00:00Z",
                    "updated_at": "2026-05-08T00:00:00Z",
                },
            }
        }
        self.runtime_tenants: dict[str, list[dict[str, Any]]] = {
            "team-request-tracker": [
                {
                    "schema_version": "cue.runtime-tenant.v0",
                    "runtime_tenant_id": "rt-team-request-tracker-sandbox-v1",
                    "app_id": "t-operations-team-request-tracker",
                    "environment": "sandbox",
                    "app_version": 1,
                    "app_spec_ref": "8a58d2e59c7f409184b85d9b46a0f45e3f28de8f",
                    "release_ref": "8a58d2e59c7f409184b85d9b46a0f45e3f28de8f",
                    "owner_namespace": "team",
                    "owner_user": "ops-lead@example.com",
                    "owner_team": "Operations",
                    "storage": {
                        "backend": "postgresql",
                        "local_backend": "postgresql",
                        "cluster_id": "local-dev-postgres",
                        "cluster_mode": "shared",
                        "isolation_unit": "database",
                        "database_name": "cue_app_t_operations_team_request_tracker_sandbox",
                        "schema_name": "app",
                        "tenant_key_columns": ["app_id", "environment", "runtime_tenant_id"],
                        "backup_required": False,
                    },
                    "data_region": "local-dev",
                    "retention": {
                        "policy_id": "sandbox_30d_resettable",
                        "archive_after_days": 30,
                        "retire_after_days": 60,
                        "export_required_before_retire": False,
                        "erase_supported": True,
                    },
                    "migration": {
                        "state": "current",
                        "last_checked_at": "2026-05-08T00:00:00Z",
                        "migration_plan_ref": None,
                    },
                    "sandbox_policy": {
                        "default_expiry_days": 30,
                        "reset_allowed": True,
                        "production_copy_allowed": False,
                    },
                    "runtime_families": [
                        "record",
                        "comment",
                        "attachment",
                        "workflow_state",
                        "dashboard_materialization",
                        "usage_metric",
                        "runtime_audit",
                    ],
                    "audit": {
                        "created_by": "cue-platform@example.com",
                        "created_at": "2026-05-08T00:00:00Z",
                        "updated_at": "2026-05-08T00:00:00Z",
                    },
                }
            ]
        }
        self.review_tickets: list[AdminReviewTicket] = [
            AdminReviewTicket(
                id="ticket-request-tracker-retention",
                project_id="team-request-tracker",
                workitem_id="request-tracker-retention",
                kind="backend_blocker",
                state="open",
                evidence_ids=["audit-request-tracker-prd-1"],
            )
        ]
        self.audit_events: list[AuditEvent] = [
            AuditEvent(
                id="audit-request-tracker-prd-1",
                workitem_id="request-tracker-prd",
                actor="cue",
                action="classified-prompt",
                created_at="2026-05-12T10:28:00Z",
                evidence={
                    "classification": "prompt-to-PRD",
                    "accepted": True,
                    "source": "fixture",
                },
            )
        ]
        self.approval_requests: list[dict[str, Any]] = []
        self.registry_snapshots: list[dict[str, Any]] = []
        self.test_history: list[dict[str, Any]] = []
        self.project_goals: dict[str, dict[str, Any]] = {
            "team-request-tracker": {
                "version": 1,
                "statement": "Reduce dropped internal support requests.",
                "problem": "Requests arrive through chat and spreadsheets, so owners lose context and managers cannot see aging work.",
                "target_users": ["requester", "operations-owner"],
                "success_metrics": [
                    {
                        "name": "request_owner_coverage",
                        "baseline": "manual spot checks",
                        "target": "100% accepted requests have owner, state, and next action",
                        "window": "30 days",
                    }
                ],
                "review_policy": {"cadence": "monthly", "owner_required": True},
                "updated_at": "2026-05-12T10:30:00Z",
                "updated_by": "cue",
            }
        }

    def list_projects(self) -> list[dict[str, Any]]:
        return [self.project_summary(project) for project in self.projects.values()]

    def get_project(self, project_id: str) -> dict[str, Any] | None:
        project = self.projects.get(project_id)
        return self.project_payload(project) if project else None

    def list_workitems(self, project_id: str) -> list[dict[str, Any]]:
        project = self.projects.get(project_id)
        return [] if project is None else [to_payload(item) for item in project.workitems]

    def list_artifacts(self, project_id: str) -> list[dict[str, Any]]:
        project = self.projects.get(project_id)
        return [] if project is None else [to_payload(artifact) for artifact in project.artifacts]

    def get_registry_entry(self, project_id: str) -> dict[str, Any] | None:
        project = self.projects.get(project_id)
        if project is None:
            return None
        repository = self.repositories.get(project_id)
        ownership = self.ownership_namespaces.get(project_id)
        runtime_tenants = self.runtime_tenants.get(project_id, [])
        return {
            "project": {
                "id": project.id,
                "name": project.name,
                "owner_namespace": project.owner_namespace,
                "risk_tier": project.risk_tier,
                "lifecycle_status": project.lifecycle_status,
            },
            "artifact_repository": to_payload(repository),
            "ownership_namespace": self.get_ownership_namespace(project_id, include_infrastructure=True),
            "runtime_tenants": deepcopy(runtime_tenants),
            "user_visible_infrastructure": False,
        }

    def app_registry_catalog(self, query: str = "", filters: dict[str, str] | None = None, include_internal: bool = False) -> list[dict[str, Any]]:
        filters = filters or {}
        rows: list[dict[str, Any]] = []
        for project in self.projects.values():
            repository = self.repositories.get(project.id)
            metrics = self.project_metrics(project.id) or {}
            health = self.project_health_score(project.id) or {"status": "healthy", "score": 100}
            connectors = [
                connector["id"]
                for connector in CONNECTOR_CATALOG.get("connectors", {}).values()
                if connector.get("data_owner")
            ] if "CONNECTOR_CATALOG" in globals() else ["request_tracker_db"]
            row = {
                "app_id": project.id,
                "name": project.name,
                "owner": project.owner,
                "owner_namespace": project.owner_namespace,
                "risk_tier": project.risk_tier,
                "lifecycle_status": "sandbox" if self.runtime_tenants.get(project.id) else project.lifecycle_status,
                "data_sources": connectors,
                "permission_summary": {"roles": ["owner", "editor", "viewer"], "sensitive_fields_masked": True},
                "current_version": 1,
                "usage": {"active_users": metrics.get("active_users", 0), "last_used_at": metrics.get("last_used_at")},
                "goal_status": self.project_goal_status(project.id),
                "goal_metrics": self.project_goal_metrics(project.id),
                "health": {"status": health["status"], "score": health["score"]},
                "hidden_repo_backed": bool(repository and repository.hidden_gitlab_project_id and (repository.current_spec_ref or repository.sandbox_ref)),
                "orphan_state": self.ownership_namespaces.get(project.id, {}).get("owner_namespace", {}).get("orphan_state", "unknown"),
                "retention_behavior": "retain_hidden_repo_audit_history",
            }
            if include_internal and repository:
                row["source_mapping"] = {
                    "gitlab_project_id": repository.hidden_gitlab_project_id,
                    "project_url_internal": repository.gitlab_full_path,
                    "default_branch": repository.default_branch,
                    "current_commit_sha": repository.current_spec_ref,
                    "sandbox_ref": repository.sandbox_ref,
                    "production_ref": repository.production_ref,
                    "latest_release_tag": repository.production_release_tag,
                }
            rows.append(row)

        normalized_query = query.strip().lower()
        if normalized_query:
            rows = [row for row in rows if normalized_query in row["name"].lower() or normalized_query in row["owner"].lower()]
        for key, value in filters.items():
            if value:
                rows = [row for row in rows if str(row.get(key)) == value or value in row.get("data_sources", [])]
        return rows

    def project_goal(self, project_id: str) -> dict[str, Any] | None:
        goal = self.project_goals.get(project_id)
        return deepcopy(goal) if goal else None

    def project_goal_metrics(self, project_id: str) -> list[dict[str, Any]]:
        goal = self.project_goals.get(project_id)
        if goal is None:
            return []
        metrics = self.project_metrics(project_id) or {}
        runtime_metrics = metrics.get("runtime_metrics", {})
        total = int(runtime_metrics.get("todo_count") or 0)
        completed = int(runtime_metrics.get("completed_count") or 0)
        completion_rate = round(completed / total, 4) if total else None
        return [
            {
                **metric,
                "current": completion_rate if metric["name"] == "request_owner_coverage" else metrics.get("active_users", 0),
                "source": "usage_and_runtime_metrics",
            }
            for metric in goal.get("success_metrics", [])
        ]

    def project_goal_status(self, project_id: str) -> dict[str, Any]:
        goal = self.project_goals.get(project_id)
        metrics = self.project_metrics(project_id) or {}
        if goal is None:
            return {"status": "missing", "reason": "goal_not_defined", "review_due": True}
        if metrics.get("error_count", 0) > 0:
            return {"status": "at_risk", "reason": "runtime_errors", "review_due": True}
        if metrics.get("active_users", 0) == 0:
            return {"status": "needs_review", "reason": "no_active_users", "review_due": True}
        return {"status": "on_track", "reason": "usage_present", "review_due": False}

    def project_goal_review(self, project_id: str) -> dict[str, Any] | None:
        if project_id not in self.projects:
            return None
        status = self.project_goal_status(project_id)
        metrics = self.project_metrics(project_id) or {}
        lifecycle_recommendation = "continue"
        if status["status"] == "missing":
            lifecycle_recommendation = "block_production"
        elif metrics.get("active_users", 0) == 0:
            lifecycle_recommendation = "review_for_archive_or_retire"
        elif status["status"] == "at_risk":
            lifecycle_recommendation = "revise_goal_or_runtime"
        return {
            "goal": self.project_goal(project_id),
            "goal_status": status,
            "goal_metrics": self.project_goal_metrics(project_id),
            "usage_metrics": metrics,
            "retirement_inputs": {
                "active_users": metrics.get("active_users", 0),
                "last_used_at": metrics.get("last_used_at"),
                "error_count": metrics.get("error_count", 0),
                "goal_review_due": status["review_due"],
            },
            "lifecycle_recommendation": lifecycle_recommendation,
        }

    def update_project_goal(self, project_id: str, goal: dict[str, Any], actor: str = "owner") -> dict[str, Any] | None:
        project = self.projects.get(project_id)
        if project is None:
            return None
        previous = self.project_goals.get(project_id)
        next_goal = deepcopy(goal)
        next_goal["version"] = int((previous or {}).get("version", 0)) + 1
        next_goal["updated_at"] = utc_now()
        next_goal["updated_by"] = actor
        self.project_goals[project_id] = next_goal
        self.append_audit(
            project.workitems[0].id,
            actor,
            "goal-updated",
            {"previous": previous, "next": next_goal, "version": next_goal["version"]},
        )
        return self.project_goal_review(project_id)

    def duplicate_app_candidates(self, name: str, owner: str | None = None) -> list[dict[str, Any]]:
        normalized = name.strip().lower()
        return [
            row
            for row in self.app_registry_catalog()
            if row["name"].strip().lower() == normalized and (owner is None or row["owner"] == owner)
        ]

    def get_ownership_namespace(self, project_id: str, include_infrastructure: bool = False) -> dict[str, Any] | None:
        namespace = self.ownership_namespaces.get(project_id)
        if namespace is None:
            return None
        payload = deepcopy(namespace)
        if not include_infrastructure:
            payload.pop("gitlab_mapping", None)
        return payload

    def list_runtime_tenants(self, project_id: str) -> list[dict[str, Any]] | None:
        if project_id not in self.projects:
            return None
        return deepcopy(self.runtime_tenants.get(project_id, []))

    def project_audit_events(self, project_id: str) -> list[dict[str, Any]] | None:
        project = self.projects.get(project_id)
        if project is None:
            return None
        workitem_ids = {workitem.id for workitem in project.workitems}
        return [to_payload(event) for event in self.audit_events if event.workitem_id in workitem_ids]

    def project_metrics(self, project_id: str) -> dict[str, Any] | None:
        project = self.projects.get(project_id)
        if project is None:
            return None
        audit_events = self.project_audit_events(project_id) or []
        dashboard = getattr(project, "operations_dashboard", None) or {}
        runtime_metrics = dashboard.get("runtime_metrics", {})
        error_events = [event for event in audit_events if event["action"] in {"error", "failed", "policy-error"}]
        workflow_events = [event for event in audit_events if event["action"] in {"classified-prompt", "submitted-prompt", "created-prd"}]
        return {
            "active_users": len({message.speaker for message in project.messages if message.speaker == "owner"}),
            "last_used_at": max([event["created_at"] for event in audit_events], default=None),
            "audit_event_count": len(audit_events),
            "error_count": len(error_events),
            "workflow_trace_count": len(workflow_events),
            "runtime_metrics": deepcopy(runtime_metrics),
        }

    def project_health_score(self, project_id: str) -> dict[str, Any] | None:
        project = self.projects.get(project_id)
        if project is None:
            return None
        metrics = self.project_metrics(project_id) or {}
        open_tickets = [ticket for ticket in self.review_tickets if ticket.project_id == project_id and ticket.state == "open"]
        score = 100
        reasons: list[str] = []
        if project.lifecycle_status in {"blocked", "archived", "retired"}:
            score -= 35
            reasons.append("lifecycle_status")
        if open_tickets:
            score -= 20
            reasons.append("open_review_tickets")
        if metrics.get("error_count", 0) > 0:
            score -= 20
            reasons.append("runtime_errors")
        if not metrics.get("last_used_at"):
            score -= 15
            reasons.append("unused")
        score = max(score, 0)
        return {
            "score": score,
            "status": "healthy" if score >= 80 else "watch" if score >= 60 else "risky",
            "reasons": reasons,
            "metrics": metrics,
        }

    def deployment_action(self, project_id: str, action: str, actor: str = "owner", target_ref: str | None = None) -> dict[str, Any] | None:
        project = self.projects.get(project_id)
        repository = self.repositories.get(project_id)
        if project is None or repository is None:
            return None
        before = {
            "lifecycle_status": project.lifecycle_status,
            "sandbox_ref": repository.sandbox_ref,
            "production_ref": repository.production_ref,
            "production_release_tag": repository.production_release_tag,
        }
        gates = {
            "schema": "passed",
            "policy": "passed",
            "permissions": "passed",
            "generated_artifacts": "passed",
            "regression_tests": "passed",
        }
        review_gate = None
        if action == "deploy_sandbox":
            project.lifecycle_status = "sandbox"
            repository.sandbox_ref = repository.current_spec_ref or "draft-ref-fixture"
            status = "sandbox_deployed"
        elif action == "request_production":
            if project.lifecycle_status != "sandbox":
                return {
                    "status": "blocked",
                    "reason": "production_requires_sandbox",
                    "gates": gates,
                }
            review_gate = {
                "state": "open",
                "required_approvals": ["app_owner", "data_owner"] if project.risk_tier in {"tier_2", "tier_3", "tier_4"} else ["app_owner"],
                "ci": gates,
            }
            status = "production_review_requested"
        elif action == "approve_production":
            if project.lifecycle_status != "sandbox":
                return {"status": "blocked", "reason": "approval_requires_sandbox", "gates": gates}
            project.lifecycle_status = "production"
            repository.production_ref = repository.sandbox_ref or repository.current_spec_ref
            repository.production_release_tag = "cue-team-request-tracker-v1"
            status = "production_deployed"
        elif action == "rollback":
            rollback_ref = target_ref or repository.production_ref or repository.sandbox_ref
            if not rollback_ref:
                return {"status": "blocked", "reason": "rollback_requires_prior_ref", "gates": gates}
            project.lifecycle_status = "production"
            repository.production_ref = rollback_ref
            repository.production_release_tag = f"{repository.production_release_tag or 'cue-release'}-rollback"
            status = "rolled_back"
        elif action == "emergency_disable":
            project.lifecycle_status = "blocked"
            status = "disabled"
        else:
            return {"status": "blocked", "reason": "unknown_deployment_action", "gates": gates}

        after = {
            "lifecycle_status": project.lifecycle_status,
            "sandbox_ref": repository.sandbox_ref,
            "production_ref": repository.production_ref,
            "production_release_tag": repository.production_release_tag,
        }
        self.append_audit(
            project.workitems[0].id,
            actor,
            f"deployment-{action}",
            {
                "app_id": project.id,
                "repo_project_id": repository.hidden_gitlab_project_id,
                "commit_sha": after["production_ref"] or after["sandbox_ref"] or repository.current_spec_ref,
                "release_tag": repository.production_release_tag,
                "before": before,
                "after": after,
                "gates": gates,
            },
        )
        return {
            "status": status,
            "lifecycle_status": project.lifecycle_status,
            "release_tag": repository.production_release_tag,
            "gates": gates,
            "review_gate": review_gate,
        }

    def governance_transition(self, project_id: str, event: str, actor: str, facts: dict[str, Any] | None = None) -> dict[str, Any] | None:
        project = self.projects.get(project_id)
        repository = self.repositories.get(project_id)
        if project is None or repository is None:
            return None
        facts = facts or {}
        blocked_actions = sorted(set(facts.get("actions") or []) & TIER_4_BLOCKLIST)
        if project.risk_tier == "tier_4" or blocked_actions:
            return {
                "status": "rejected",
                "reason": "tier_4_blocklist",
                "blocked_actions": blocked_actions,
                "lifecycle_status": project.lifecycle_status,
            }

        before = self.registry_snapshot(project_id)
        approval_request = None
        next_status = project.lifecycle_status
        audit_type = event

        if event == "validate_spec":
            required = {"app_spec_schema_valid", "owner_user_present", "owner_team_present", "target_users_present", "at_least_one_entity"}
            missing = sorted(required - set(facts.get("satisfied") or []))
            if missing:
                return {"status": "rejected", "reason": "missing_preconditions", "missing": missing}
            next_status = "active"
            audit_type = "app_spec_validated"
        elif event == "deploy_sandbox":
            required = {"risk_tier_assigned", "policy_check_passed", "required_tests_passed", "tier_4_blocklist_not_matched"}
            missing = sorted(required - set(facts.get("satisfied") or []))
            if missing:
                return {"status": "rejected", "reason": "missing_preconditions", "missing": missing}
            next_status = "sandbox"
            repository.sandbox_ref = facts.get("commit_sha") or repository.current_spec_ref
            audit_type = "sandbox_deployed"
        elif event == "request_production":
            if project.lifecycle_status != "sandbox":
                return {"status": "rejected", "reason": "production_requires_sandbox"}
            approval_request = self.open_approval_request(project, actor)
            audit_type = "production_requested"
        elif event == "approve_production":
            if not self.production_approvals_satisfied(project_id):
                return {"status": "rejected", "reason": "approval_required"}
            next_status = "production"
            repository.production_ref = facts.get("commit_sha") or repository.sandbox_ref or repository.current_spec_ref
            repository.production_release_tag = facts.get("release_tag") or f"{project.id}-v1"
            audit_type = "production_deployed"
        elif event == "emergency_disable":
            next_status = "blocked"
            audit_type = "app_disabled"
        elif event == "archive":
            next_status = "archived"
            audit_type = "app_archived"
        elif event == "retire":
            next_status = "retired"
            audit_type = "app_retired"
        else:
            return {"status": "rejected", "reason": "unknown_transition"}

        project.lifecycle_status = next_status  # type: ignore[assignment]
        after = self.registry_snapshot(project_id)
        if next_status in {"sandbox", "production", "blocked", "archived", "retired"}:
            self.registry_snapshots.append(after)
        self.append_audit(
            project.workitems[0].id,
            actor,
            audit_type,
            {
                "actor": actor,
                "app_id": project.id,
                "app_version": 1,
                "event_type": audit_type,
                "before": before,
                "after": after,
                "commit_sha": facts.get("commit_sha") or repository.production_ref or repository.sandbox_ref or repository.current_spec_ref,
                "release_tag": repository.production_release_tag,
            },
        )
        return {
            "status": "applied",
            "event_type": audit_type,
            "lifecycle_status": project.lifecycle_status,
            "approval_request": approval_request,
            "registry_snapshot": after,
        }

    def open_approval_request(self, project: Project, actor: str) -> dict[str, Any]:
        required = [{"kind": "app_owner", "principal": project.owner, "decision": "pending", "comment": None}]
        if project.risk_tier in {"tier_2", "tier_3", "tier_4"}:
            required.append({"kind": "data_owner", "principal": "operations@example.com", "decision": "pending", "comment": None})
        request = {
            "id": str(uuid4()),
            "app_id": project.id,
            "app_version": 1,
            "risk_tier": project.risk_tier,
            "requested_by": actor,
            "required_approvers": required,
            "status": "pending",
        }
        self.approval_requests.append(request)
        return deepcopy(request)

    def production_approvals_satisfied(self, project_id: str) -> bool:
        requests = [request for request in self.approval_requests if request["app_id"] == project_id]
        if not requests:
            return False
        latest = requests[-1]
        for approver in latest["required_approvers"]:
            approver["decision"] = "approved"
        latest["status"] = "approved"
        return True

    def registry_snapshot(self, project_id: str) -> dict[str, Any]:
        project = self.projects[project_id]
        repository = self.repositories.get(project_id)
        health = self.project_health_score(project_id) or {"status": "healthy"}
        return {
            "app_id": project.id,
            "app_version": 1,
            "name": project.name,
            "owner_team": project.owner,
            "owner_user": "ops-lead@example.com",
            "risk_tier": project.risk_tier,
            "lifecycle_status": "disabled" if project.lifecycle_status == "blocked" else project.lifecycle_status,
            "goal_status": self.project_goal_status(project_id),
            "goal_metrics": self.project_goal_metrics(project_id),
            "deployment_environment": "production" if project.lifecycle_status == "production" else "sandbox",
            "repo_project_id": repository.hidden_gitlab_project_id if repository else None,
            "health": {
                "status": health["status"],
                "last_test_run_status": "passed",
                "open_policy_findings": len([ticket for ticket in self.review_tickets if ticket.project_id == project_id and ticket.state == "open"]),
            },
        }

    def hidden_repo_template_manifest(self) -> dict[str, Any]:
        return {
            "schema_version": "cue.hidden-repo-template.v0",
            "template_root": "projects/cue/app-repo-template",
            "visibility": "hidden",
            "user_visible": False,
            "required_files": list(HIDDEN_REPO_TEMPLATE_FILES),
            "mvp_generated_code_required": False,
            "provisioner": {
                "adapter": "fixture",
                "path_source": "ownership_namespace.gitlab_mapping.full_path",
                "registry_writer": "artifact_repository",
            },
        }

    def template_library_manifest(self) -> dict[str, Any]:
        template_path = CUE_ROOT / "examples" / "template-library.v0.json"
        return json.loads(template_path.read_text())

    def pilot_acceptance_dashboard(self) -> dict[str, Any]:
        manifest = self.template_library_manifest()
        templates = manifest["templates"]
        active_apps = len(templates)
        return {
            "schema_version": "cue.pilot-acceptance-dashboard.v0",
            "pilot": manifest["pilot"],
            "metrics": {
                "active_apps": active_apps,
                "active_users": sum(template["pilot_seed"]["active_users"] for template in templates),
                "policy_pass_rate": round(
                    sum(1 for template in templates if template["pilot_seed"]["policy_status"] == "pass") / active_apps,
                    4,
                ),
                "test_pass_rate": round(
                    sum(template["pilot_seed"]["tests_passed"] for template in templates)
                    / sum(template["pilot_seed"]["tests_total"] for template in templates),
                    4,
                ),
                "deployment_status": {
                    status: sum(1 for template in templates if template["pilot_seed"]["deployment_status"] == status)
                    for status in sorted({template["pilot_seed"]["deployment_status"] for template in templates})
                },
                "incidents": sum(template["pilot_seed"]["incidents"] for template in templates),
                "app_owner_satisfaction": round(
                    sum(template["pilot_seed"]["owner_satisfaction"] for template in templates) / active_apps,
                    2,
                ),
            },
            "template_ids": [template["id"] for template in templates],
        }

    def product_layout_manifest(self) -> dict[str, Any]:
        return {
            "schema_version": "cue.product-layout.v0",
            "legacy_retirement": {
                "status": "retired",
                "retirement_doc": "projects/cue/docs/legacy/RETIREMENT.md",
                "closed_issues": [1243, 1245, 1246, 1247, 1248, 1226],
                "transitional_paths": ["projects/cue/app", "projects/cue/fe", "projects/cue/be"],
            },
            "session_boundary": {
                "auth_mode": "placeholder",
                "api_base_path": "/api",
                "owner_role": "project_owner",
                "admin_role": "platform_operator",
            },
            "workspaces": [
                {"name": "artifact_studio", "path": "projects/cue/artifact-studio", "audience": "project_owner", "role": "frontend_site"},
                {"name": "admin", "path": "projects/cue/admin", "audience": "platform_operator", "role": "frontend_site"},
                {"name": "backend", "path": "projects/cue/backend", "audience": "developer", "role": "api_service"},
                {"name": "shared", "path": "projects/cue/shared", "audience": "developer", "role": "shared_contracts"},
                {"name": "schemas", "path": "projects/cue/schemas", "audience": "developer", "role": "contract_store"},
                {"name": "examples", "path": "projects/cue/examples", "audience": "generated_app_runtime", "role": "fixture_store"},
                {"name": "legacy_docs", "path": "projects/cue/docs/legacy", "audience": "developer", "role": "history_only"},
            ],
        }

    def frontend_shell_manifest(self) -> dict[str, Any]:
        return {
            "schema_version": "cue.react-on-jet-shell.v0",
            "source_spec": ".aw/tech-design/projects/cue/react-on-jet-frontend-shell.md",
            "sites": [
                {
                    "name": "artifact_studio",
                    "path": "projects/cue/artifact-studio",
                    "audience": "project_owner",
                    "ui_runtime": "react_tsx",
                    "entrypoint": "src/main.tsx",
                },
                {
                    "name": "admin",
                    "path": "projects/cue/admin",
                    "audience": "platform_operator",
                    "ui_runtime": "react_tsx",
                    "entrypoint": "src/main.tsx",
                },
            ],
            "substrate": {
                "owner": "jet",
                "target_package_manager": "jet",
                "current_bridge": "vite",
                "bridge_reason": "jet_cli_not_available_in_this_checkout",
                "cue_workaround_policy": "narrow_and_removable",
            },
            "validation": {
                "artifact_studio": {
                    "typecheck": "npm run typecheck",
                    "build": "npm run build",
                    "e2e": "npm run test:e2e",
                },
                "admin": {
                    "typecheck": "npm run typecheck",
                    "build": "npm run build",
                },
                "jet_blocker_policy": "If Vite validates a React TSX slice and Jet fails, file or link a project:jet issue before adding a Cue-side workaround.",
            },
        }

    def artifact_graph_manifest(self) -> dict[str, Any]:
        return {
            "schema_version": "cue.artifact-graph.v0",
            "artifact_types": [
                {"kind": "workitem", "owner": "project_owner", "review_required": False},
                {"kind": "prd", "owner": "project_owner", "review_required": True},
                {"kind": "td", "owner": "agent_team", "review_required": True},
                {"kind": "app_spec", "owner": "platform", "review_required": True},
                {"kind": "runtime_manifest", "owner": "platform", "review_required": False},
            ],
            "dependencies": [
                {"from": "workitem", "to": "prd", "gate": "accepted"},
                {"from": "prd", "to": "td", "gate": "approved"},
                {"from": "td", "to": "app_spec", "gate": "approved"},
                {"from": "app_spec", "to": "runtime_manifest", "gate": "validated"},
                {"from": "app_spec", "to": "runtime_manifest", "gate": "policy_passed"},
            ],
        }

    def artifact_gates_for(self, workitem: WorkItem) -> list[dict[str, Any]]:
        project = self.projects[workitem.project_id]
        prd_artifact = next((artifact for artifact in project.artifacts if artifact.kind == "prd"), None)
        prd_approved = bool(prd_artifact and prd_artifact.status == "approved")
        td_approved = any(artifact.kind == "td" and artifact.status == "approved" for artifact in project.artifacts)
        app_spec_validated = any(artifact.kind == "app_spec" and artifact.status == "validated" for artifact in project.artifacts)
        policy_passed = any(event.action == "policy_check_completed" and event.evidence.get("result") == "pass" for event in self.audit_events)
        return [
            {
                "kind": "prd",
                "unlocked": workitem.state == "accepted",
                "gate": "accepted",
                "reason": "WorkItem must be accepted before PRD generation.",
            },
            {
                "kind": "td",
                "unlocked": prd_approved,
                "gate": "approved",
                "reason": "PRD must be approved before TD generation.",
            },
            {
                "kind": "runtime_manifest",
                "unlocked": td_approved and app_spec_validated and policy_passed,
                "gate": "validated+policy_passed",
                "reason": "Runtime requires approved TD, validated App Spec, and passing policy.",
            },
        ]

    def resolve_workspace(self, audience: str) -> dict[str, str]:
        routing = {
            "owner": "artifact_studio",
            "project_owner": "artifact_studio",
            "operator": "admin",
            "platform_operator": "admin",
            "backend": "backend",
            "api": "backend",
            "contract": "shared",
            "schema": "schemas",
            "legacy": "legacy_docs",
        }
        target = routing.get(audience, "legacy_docs")
        for workspace in self.product_layout_manifest()["workspaces"]:
            if workspace["name"] == target:
                return workspace
        return self.product_layout_manifest()["workspaces"][-1]

    def project_for_session(self, session_id: str) -> Project | None:
        for project in self.projects.values():
            if self.session_id(project) == session_id:
                return project
        return None

    def get_workitem(self, workitem_id: str) -> WorkItem | None:
        for project in self.projects.values():
            for workitem in project.workitems:
                if workitem.id == workitem_id:
                    return workitem
        return None

    def replace_workitem(self, workitem: WorkItem) -> None:
        project = self.projects[workitem.project_id]
        project.workitems = [workitem if item.id == workitem.id else item for item in project.workitems]
        self.pgkit.execute(
            pg_update(
                "cue_workitems",
                {
                    "title": workitem.title,
                    "state": workitem.state,
                    "next_action": workitem.next_action,
                    "target_artifact_type": workitem.target_artifact_type,
                    "artifact_id": workitem.artifact_id,
                    "updated_at": workitem.updated_at,
                },
                {"id": workitem.id},
            )
        )

    def append_message(self, project_id: str, speaker: Literal["cue", "owner"], body: str, action: str | None = None) -> Message:
        project = self.projects[project_id]
        message = Message(f"m{len(project.messages) + 1}", speaker, body, action)
        project.messages.append(message)
        self.pgkit.execute(
            pg_insert(
                "cue_session_messages",
                {
                    "id": message.id,
                    "project_id": project_id,
                    "session_id": self.session_id(project),
                    "speaker": message.speaker,
                    "body": message.body,
                    "action": message.action,
                },
            )
        )
        return message

    def append_workitem(self, project_id: str, workitem: WorkItem) -> None:
        project = self.projects[project_id]
        project.workitems.append(workitem)
        project.summary = "New WorkItem created from prompt."
        project.next_action = workitem.next_action
        project.status = "in-progress"
        self.pgkit.execute(
            pg_insert(
                "cue_workitems",
                {
                    "id": workitem.id,
                    "project_id": project_id,
                    "title": workitem.title,
                    "route": workitem.route,
                    "target_artifact_type": workitem.target_artifact_type,
                    "state": workitem.state,
                    "next_action": workitem.next_action,
                    "updated_at": workitem.updated_at,
                },
            )
        )

    def append_artifact_version(self, project_id: str, artifact_id: str, version: ArtifactVersion) -> dict[str, Any]:
        project = self.projects[project_id]
        for artifact in project.artifacts:
            if artifact.id == artifact_id:
                artifact.versions.append(version)
                artifact.status = version.status
                self.persist_artifact_version(project_id, artifact_id, version)
                return to_payload(artifact)
        artifact = Artifact(artifact_id, "Project PRD", "prd", version.status, [version])
        project.artifacts.append(artifact)
        self.persist_artifact_version(project_id, artifact_id, version)
        return to_payload(artifact)

    def append_audit(self, workitem_id: str, actor: str, action: str, evidence: dict[str, Any]) -> AuditEvent:
        event = AuditEvent(
            id=f"audit-{workitem_id}-{len(self.audit_events) + 1}",
            workitem_id=workitem_id,
            actor=actor,
            action=action,
            created_at=utc_now(),
            evidence=deepcopy(evidence),
        )
        self.audit_events.append(event)
        self.pgkit.execute(
            pg_insert(
                "cue_audit_events",
                {
                    "id": event.id,
                    "workitem_id": event.workitem_id,
                    "actor": event.actor,
                    "action": event.action,
                    "created_at": event.created_at,
                    "evidence": event.evidence,
                },
            )
        )
        return event

    def persist_artifact_version(self, project_id: str, artifact_id: str, version: ArtifactVersion) -> None:
        self.pgkit.execute(
            pg_insert(
                "cue_artifact_versions",
                {
                    "id": version.id,
                    "project_id": project_id,
                    "artifact_id": artifact_id,
                    "created_at": version.created_at,
                    "status": version.status,
                    "summary": version.summary,
                    "body": version.body,
                },
            )
        )

    def list_admin_workitems(self) -> list[dict[str, Any]]:
        rows: list[dict[str, Any]] = []
        for project in self.projects.values():
            for workitem in project.workitems:
                payload = to_payload(workitem)
                payload["project_name"] = project.name
                payload["audit_count"] = len([event for event in self.audit_events if event.workitem_id == workitem.id])
                payload["review_ticket_count"] = len(
                    [ticket for ticket in self.review_tickets if ticket.workitem_id == workitem.id and ticket.state == "open"]
                )
                rows.append(payload)
        return rows

    def evidence_for_workitem(self, workitem_id: str) -> dict[str, Any] | None:
        workitem = self.get_workitem(workitem_id)
        if workitem is None:
            return None
        project = self.projects[workitem.project_id]
        tickets = [ticket for ticket in self.review_tickets if ticket.workitem_id == workitem_id]
        events = [event for event in self.audit_events if event.workitem_id == workitem_id]
        return {
            "workitem": to_payload(workitem),
            "project": {
                "id": project.id,
                "name": project.name,
                "owner_namespace": project.owner_namespace,
                "risk_tier": project.risk_tier,
                "lifecycle_status": project.lifecycle_status,
            },
            "artifact_repository": to_payload(self.repositories.get(workitem.project_id)),
            "ownership_namespace": self.get_ownership_namespace(workitem.project_id),
            "runtime_tenants": self.list_runtime_tenants(workitem.project_id),
            "review_tickets": [to_payload(ticket) for ticket in tickets],
            "events": [to_payload(event) for event in events],
            "audit_event_ids": [event.id for event in events],
            "blockers": list(workitem.blockers or workitem.missing_fields),
            "diagnostics": {
                "risk_hints": list(workitem.risk_hints),
                "requires_admin_review": bool(tickets or workitem.blockers),
            },
        }

    def project_summary(self, project: Project) -> dict[str, Any]:
        payload = self.project_payload(project)
        payload["messages"] = payload["messages"][-1:]
        payload["sessions"] = [self.session_payload(project, messages=project.messages[-1:])]
        return payload

    def project_payload(self, project: Project) -> dict[str, Any]:
        payload = to_payload(project)
        payload["active_session_id"] = self.session_id(project)
        payload["sessions"] = [self.session_payload(project)]
        payload["workitems"] = [self.workitem_payload(workitem) for workitem in project.workitems]
        payload["goal"] = self.project_goal(project.id)
        payload["goal_status"] = self.project_goal_status(project.id)
        payload["goal_metrics"] = self.project_goal_metrics(project.id)
        return payload

    def session_id(self, project: Project) -> str:
        return f"session-{project.id}"

    def session_payload(self, project: Project, messages: list[Message] | None = None) -> dict[str, Any]:
        return {
            "id": self.session_id(project),
            "project_id": project.id,
            "title": f"{project.name} intake",
            "messages": [to_payload(message) for message in (messages if messages is not None else project.messages)],
        }

    def workitem_payload(self, workitem: WorkItem) -> dict[str, Any]:
        payload = to_payload(workitem)
        payload["workflow_plan"] = self.workflow_plan_for(workitem)
        payload["qc_status"] = "needs_input" if workitem.blockers or workitem.missing_fields else "pass"
        payload["qc_checks"] = self.qc_checks_for(workitem)
        return payload

    def workflow_plan_for(self, workitem: WorkItem) -> list[dict[str, Any]]:
        if workitem.state == "collecting":
            prd_state = "blocked"
        elif workitem.artifact_id:
            prd_state = "in-progress"
        else:
            prd_state = "ready"
        if workitem.workflow_plan:
            plan = deepcopy(workitem.workflow_plan)
            for step in plan:
                step.update(WORKFLOW_NODE_AGENTS.get(str(step.get("id") or ""), {}))
                if step.get("id") == "prd":
                    step["state"] = prd_state
            return plan
        return [
            workflow_step("prd", "PRD", prd_state, []),
            workflow_step("td", "TD", "not-started", ["prd"]),
            workflow_step("website", "Website", "not-started", ["td"]),
        ]

    def qc_checks_for(self, workitem: WorkItem) -> list[dict[str, str]]:
        blockers = workitem.blockers or workitem.missing_fields
        if blockers:
            return [
                {
                    "id": "missing_fields",
                    "label": "Missing fields",
                    "status": "needs_input",
                    "summary": f"Needs {', '.join(blockers)}.",
                }
            ]
        return [
            {
                "id": "intent",
                "label": "Intent is project work",
                "status": "pass",
                "summary": "Prompt maps to a governed WorkItem.",
            }
        ]

    def clone_workitem(self, workitem: WorkItem, **changes: Any) -> WorkItem:
        return replace(workitem, **changes, updated_at=utc_now())


store = WorkstreamStore()
