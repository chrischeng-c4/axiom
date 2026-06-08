"""AgentKit-shaped bridge for Cue's temporary Python backend.

This module is intentionally small. It mirrors the future `cclab-agent-mamba`
task/result boundary so Cue product code can move before the Mamba binding is
ready, without growing a second agent framework in Python.
"""

from __future__ import annotations

import os
import re
import json
import subprocess
from pathlib import Path
from shutil import which
from typing import Any, Literal, Protocol, TypedDict

AgentStage = Literal["prd", "td", "codebase", "test", "deployment", "operation"]
AgentRole = Literal["pm", "architect", "designer", "dev", "data", "qa_policy", "release"]
AgentStatus = Literal["completed", "needs_input", "blocked", "failed"]

AGENT_PROVIDER_ENV = "CUE_AGENT_PROVIDER"
DETERMINISTIC_PROVIDER = "deterministic"
CLAUDE_HEADLESS_PROVIDER = "claude_headless"
CLAUDE_COMMAND_ENV = "CUE_CLAUDE_COMMAND"
CLAUDE_TIMEOUT_ENV = "CUE_CLAUDE_TIMEOUT_SECONDS"

GENERAL_CHAT_PATTERNS = (
    r"\bweather\b",
    r"\bjoke\b",
    r"\bhello\b",
    r"\bhi\b",
    r"天氣",
    r"笑話",
    r"聊天",
)

UNSUPPORTED_CONTROL_BYPASS_PATTERNS = (
    r"\bbypass\b.*\b(audit|approval|authorization|policy|review)\b",
    r"\bskip\b.*\b(audit|approval|authorization|policy|review)\b",
    r"\bdisable\b.*\b(audit|approval|authorization|policy|review)\b",
    r"\bwithout\b.*\b(audit|approval|authorization|policy|review)\b",
    r"\bignore\b.*\b(data owner|policy|approval|audit)\b",
    r"避開.*(審計|審核|核准|授權|政策|資料負責人)",
    r"跳過.*(審計|審核|核准|授權|政策|資料負責人)",
    r"不要.*(審計|審核|核准|授權|政策|資料負責人)",
)


class AgentTask(TypedDict, total=False):
    id: str
    workitem_id: str
    stage_id: AgentStage
    task_id: str
    role: AgentRole
    prompt: str
    context: dict[str, Any]
    output_schema: dict[str, Any]
    provider_hint: Literal["mambalibs", "agentkit_mamba", "deterministic", "claude_headless"]


class AgentResult(TypedDict):
    task_id: str
    status: AgentStatus
    content: dict[str, Any]
    artifact_refs: list[dict[str, Any]]
    review_tickets: list[dict[str, Any]]
    error: str | None


class AgentRunner(Protocol):
    """Minimal provider boundary that future Mamba bindings must satisfy."""

    def run(self, task: AgentTask) -> AgentResult:
        """Run a structured agent task and return a structured result."""


class DeterministicAgentRunner:
    """Rule-based runner for local development and contract tests."""

    def run(self, task: AgentTask) -> AgentResult:
        prompt = str(task.get("prompt", "")).strip()
        content = classify_workitem_prompt(prompt)
        status: AgentStatus = "completed" if content.get("accepted") else "needs_input"
        return {
            "task_id": str(task.get("task_id") or "classify_prompt"),
            "status": status,
            "content": content,
            "artifact_refs": [],
            "review_tickets": [],
            "error": None,
        }


class ClaudeHeadlessAgentRunner:
    """Agent runner backed by Claude Code headless: `claude -p <prompt>`."""

    def __init__(self, command: str | None = None, timeout_seconds: int | None = None) -> None:
        self.command = command or os.getenv(CLAUDE_COMMAND_ENV) or resolve_claude_command()
        timeout = timeout_seconds or int(os.getenv(CLAUDE_TIMEOUT_ENV, "120"))
        self.timeout_seconds = max(timeout, 1)

    def run(self, task: AgentTask) -> AgentResult:
        prompt = render_claude_prompt(task)
        try:
            completed = subprocess.run(
                [self.command, "-p", prompt],
                capture_output=True,
                check=False,
                text=True,
                timeout=self.timeout_seconds,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            return failed_result(task, str(error))

        if completed.returncode != 0:
            error = completed.stderr.strip() or f"claude exited with {completed.returncode}"
            return failed_result(task, error)

        return parse_agent_result(task, completed.stdout)


def get_agent_runner() -> AgentRunner:
    """Return the configured local AgentKit-compatible runner."""

    provider = os.getenv(AGENT_PROVIDER_ENV, DETERMINISTIC_PROVIDER).strip().lower()
    if provider in {"", DETERMINISTIC_PROVIDER, "local", "fixture", "mambalibs"}:
        return DeterministicAgentRunner()
    if provider in {CLAUDE_HEADLESS_PROVIDER, "claude", "claude-code", "claude_code"}:
        return ClaudeHeadlessAgentRunner()
    raise RuntimeError(
        f"Unsupported {AGENT_PROVIDER_ENV}={provider!r}; supported providers are deterministic and claude_headless"
    )


def classify_workitem_prompt(prompt: str) -> dict[str, Any]:
    """Return Cue's prompt-to-WorkItem routing payload."""

    if not prompt:
        return {
            "classification": "needs-clarification",
            "action": "ask_for_prompt",
            "accepted": False,
            "missing_fields": ["prompt"],
        }

    lowered = prompt.lower()
    if any(re.search(pattern, lowered) for pattern in GENERAL_CHAT_PATTERNS):
        return {
            "classification": "general_chat",
            "action": "redirect",
            "accepted": False,
            "message": "Cue only turns product/workflow requests into governed artifacts.",
        }

    if any(re.search(pattern, lowered) for pattern in UNSUPPORTED_CONTROL_BYPASS_PATTERNS):
        return {
            "classification": "unsupported_request",
            "action": "block",
            "accepted": False,
            "missing_fields": [],
            "risk_hints": ["governance_control_bypass"],
            "message": "Cue cannot create artifacts that bypass audit, approval, authorization, policy, or data-owner review.",
        }

    missing_fields: list[str] = []
    if not re.search(r"\b(owner|ops|operations|admin|team|主管|負責|同事)\b", lowered):
        missing_fields.append("owner")
    if not re.search(r"\b(data|field|source|資料|欄位|來源|保存)\b", lowered):
        missing_fields.append("data_boundary")

    if missing_fields:
        return {
            "classification": "prompt-to-WorkItem",
            "action": "collect_missing_fields",
            "accepted": True,
            "missing_fields": missing_fields,
            "workflow_plan": default_workflow_plan("blocked"),
        }

    return {
        "classification": "prompt-to-PRD",
        "action": "create_workitem",
        "accepted": True,
        "missing_fields": [],
        "workflow_plan": default_workflow_plan("ready"),
    }


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
    "website": {
        "agent_role": "designer",
        "agent_label": "Designer agent",
        "agent_task": "Produce the owner-facing website artifact after TD approval.",
    },
}


def default_workflow_plan(first_state: str) -> list[dict[str, Any]]:
    return enrich_workflow_plan(
        [
            {"id": "prd", "label": "PRD", "state": first_state, "depends_on": []},
            {"id": "td", "label": "TD", "state": "not-started", "depends_on": ["prd"]},
            {"id": "codebase", "label": "Codebase", "state": "not-started", "depends_on": ["td"]},
            {"id": "test", "label": "Test", "state": "not-started", "depends_on": ["codebase"]},
            {"id": "deployment", "label": "Deployment", "state": "not-started", "depends_on": ["test"]},
            {"id": "operation", "label": "Operation", "state": "not-started", "depends_on": ["deployment"]},
        ]
    )


def enrich_workflow_plan(plan: list[dict[str, Any]]) -> list[dict[str, Any]]:
    enriched: list[dict[str, Any]] = []
    for step in plan:
        step_id = str(step.get("id") or "")
        enriched.append({**step, **WORKFLOW_NODE_AGENTS.get(step_id, {})})
    return enriched


def render_claude_prompt(task: AgentTask) -> str:
    return "\n".join(
        [
            "You are Cue's temporary AgentKit-compatible headless agent.",
            "Return only compact JSON. Do not include markdown fences.",
            "The JSON must match this shape:",
            '{"task_id":"...","status":"completed|needs_input|blocked|failed","content":{},"artifact_refs":[],"review_tickets":[],"error":null}',
            "For output_schema.type=cue.prompt-classification.v0, content must include:",
            "classification, action, accepted, missing_fields, and workflow_plan when relevant.",
            "General chat must use classification=general_chat and accepted=false.",
            "Project work with enough owner/team and data/field/source context must use classification=prompt-to-PRD and action=create_workitem.",
            "Project work missing required context must use classification=prompt-to-WorkItem, action=collect_missing_fields, and list missing_fields.",
            "workflow_plan must be an array of {id,label,state,depends_on,agent_role,agent_label,agent_task}.",
            "Use PRD -> TD -> Codebase -> Test -> Deployment -> Operation, with a different agent_role on every node.",
            "Task:",
            json.dumps(task, ensure_ascii=False, sort_keys=True),
        ]
    )


def parse_agent_result(task: AgentTask, output: str) -> AgentResult:
    raw = output.strip()
    try:
        parsed = json.loads(raw)
    except json.JSONDecodeError:
        return failed_result(task, "claude returned non-JSON output", {"raw_output": raw})

    content = parsed.get("content")
    if not isinstance(content, dict):
        return failed_result(task, "claude result missing object content", {"raw_output": raw})
    content = normalize_agent_content(task, content)

    status = parsed.get("status") or "completed"
    if status not in {"completed", "needs_input", "blocked", "failed"}:
        status = "failed"

    return {
        "task_id": str(parsed.get("task_id") or task.get("task_id") or task.get("id") or "agent_task"),
        "status": status,
        "content": content,
        "artifact_refs": list(parsed.get("artifact_refs") or []),
        "review_tickets": list(parsed.get("review_tickets") or []),
        "error": parsed.get("error"),
    }


def failed_result(task: AgentTask, error: str, content: dict[str, Any] | None = None) -> AgentResult:
    return {
        "task_id": str(task.get("task_id") or task.get("id") or "agent_task"),
        "status": "failed",
        "content": content or {},
        "artifact_refs": [],
        "review_tickets": [],
        "error": error,
    }


def resolve_claude_command() -> str:
    found = which("claude")
    if found:
        return found
    local = Path.home() / ".local" / "bin" / "claude"
    if local.exists():
        return str(local)
    return "claude"


def normalize_agent_content(task: AgentTask, content: dict[str, Any]) -> dict[str, Any]:
    if task.get("output_schema", {}).get("type") != "cue.prompt-classification.v0":
        return content

    classification = content.get("classification")
    if classification == "prompt-to-PRD":
        content["action"] = "create_workitem"
        content["accepted"] = True
        content["missing_fields"] = list(content.get("missing_fields") or [])
        if not isinstance(content.get("workflow_plan"), list):
            content["workflow_plan"] = default_workflow_plan("ready")
        else:
            content["workflow_plan"] = enrich_workflow_plan(content["workflow_plan"])
    elif classification == "prompt-to-WorkItem":
        content["action"] = "collect_missing_fields"
        content["accepted"] = True
        content["missing_fields"] = list(content.get("missing_fields") or [])
        if not isinstance(content.get("workflow_plan"), list):
            content["workflow_plan"] = default_workflow_plan("blocked")
        else:
            content["workflow_plan"] = enrich_workflow_plan(content["workflow_plan"])
    elif classification == "general_chat":
        content["action"] = "redirect"
        content["accepted"] = False
        content["missing_fields"] = list(content.get("missing_fields") or [])
    return content
