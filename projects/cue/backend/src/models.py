"""Typed payloads for the Cue Artifact Studio workstream API."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
from typing import Any, Literal

ProjectStatus = Literal["needs-review", "in-progress", "ready", "blocked"]
OwnerNamespace = Literal["personal", "team", "cross_team", "platform"]
RiskTier = Literal["tier_0", "tier_1", "tier_2", "tier_3", "tier_4"]
LifecycleStatus = Literal["draft", "active", "blocked", "sandbox", "production", "archived", "retired"]
ArtifactStage = Literal["done", "in-progress", "not-started", "blocked", "ready"]
WorkItemState = Literal["collecting", "accepted", "drafting", "blocked", "done"]
Route = Literal["prompt-to-WorkItem", "prompt-to-PRD", "prompt-to-runtime"]
AgentRole = Literal["pm", "architect", "designer", "dev", "data", "qa_policy", "release"]


def utc_now() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


@dataclass
class Stage:
    id: str
    label: str
    state: ArtifactStage
    detail: str
    agent_role: AgentRole | None = None
    agent_label: str | None = None
    agent_task: str | None = None


@dataclass
class Message:
    id: str
    speaker: Literal["cue", "owner"]
    body: str
    action: str | None = None


@dataclass
class ArtifactVersion:
    id: str
    created_at: str
    status: str
    summary: str
    body: dict[str, Any]


@dataclass
class Artifact:
    id: str
    label: str
    kind: str
    status: str
    versions: list[ArtifactVersion] = field(default_factory=list)
    entrypoints: list[str] | None = None


@dataclass
class WorkItem:
    id: str
    project_id: str
    title: str
    route: Route
    target: Literal["WorkItem", "PRD", "TD", "Runtime"]
    state: WorkItemState
    progress: int
    next_action: str
    prompt: str = ""
    missing_fields: list[str] = field(default_factory=list)
    blockers: list[str] = field(default_factory=list)
    risk_hints: list[str] = field(default_factory=list)
    workflow_plan: list[dict[str, Any]] = field(default_factory=list)
    target_artifact_type: str = "workitem"
    artifact_id: str | None = None
    updated_at: str = field(default_factory=utc_now)


@dataclass
class AuditEvent:
    id: str
    workitem_id: str
    actor: str
    action: str
    created_at: str
    evidence: dict[str, Any]


@dataclass
class ArtifactRepository:
    id: str
    project_id: str
    backend: Literal["fixture", "local", "gitlab"]
    status: Literal["not_provisioned", "fixture_backed", "provisioned", "blocked"]
    gitlab_full_path: str | None = None
    default_branch: str = "main"
    template_version: str = "fixture-v1"
    visibility: Literal["private"] = "private"
    user_visible: bool = False
    hidden_gitlab_project_id: int | None = None
    current_spec_ref: str | None = None
    sandbox_ref: str | None = None
    production_ref: str | None = None
    production_release_tag: str | None = None
    archived_at: str | None = None
    retired_at: str | None = None


@dataclass
class AdminReviewTicket:
    id: str
    project_id: str
    workitem_id: str
    kind: Literal["policy_exception", "connector_grant", "production_request", "backend_blocker"]
    state: Literal["open", "approved", "rejected", "change_requested", "closed"]
    evidence_ids: list[str]


@dataclass
class Project:
    id: str
    name: str
    owner: str
    owner_namespace: OwnerNamespace
    risk_tier: RiskTier
    lifecycle_status: LifecycleStatus
    current_workstream_id: str | None
    status: ProjectStatus
    next_action: str
    summary: str
    stages: list[Stage]
    messages: list[Message]
    artifacts: list[Artifact]
    workitems: list[WorkItem]


def to_payload(value: Any) -> Any:
    if hasattr(value, "__dataclass_fields__"):
        return asdict(value)
    if isinstance(value, list):
        return [to_payload(item) for item in value]
    if isinstance(value, dict):
        return {key: to_payload(item) for key, item in value.items()}
    return value
