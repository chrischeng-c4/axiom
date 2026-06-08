"""Generation provider boundary for Cue.

Local tests use the deterministic provider so no LLM/network/runtime dependency
is required. Runtime providers can later implement the same small surface.
"""

import json
import os
import re

GENERATOR_ENV = "CUE_LLM_PROVIDER"
DETERMINISTIC_PROVIDER = "deterministic"


def get_generator():
    """Return the configured generation provider."""
    provider = os.getenv(GENERATOR_ENV, DETERMINISTIC_PROVIDER).strip().lower()
    if provider in {"", DETERMINISTIC_PROVIDER, "local", "fixture"}:
        return DeterministicGenerator()
    raise RuntimeError(
        f"Unsupported {GENERATOR_ENV}={provider!r}; local tests must use deterministic"
    )


class DeterministicGenerator:
    """Rule-based generation for local development and e2e tests."""

    def classify_prompt(self, content: str) -> dict:
        lowered = content.lower()
        general_terms = ["hello", "hi", "weather", "joke", "chat", "你好", "哈囉", "聊天", "天氣"]
        artifact_terms = [
            "website",
            "site",
            "app",
            "prd",
            "td",
            "artifact",
            "workflow",
            "網站",
            "流程",
            "產生",
            "建立",
        ]
        is_general = any(term in lowered or term in content for term in general_terms) and not any(
            term in lowered or term in content for term in artifact_terms
        )
        if is_general:
            return {
                "classification": "general_chat_redirect",
                "reply": "這看起來是一般聊天，不會建立 WorkItem。請描述一個專案目標、工作流程或要產生的 artifact。",
            }

        target = "Website" if any(term in lowered or term in content for term in ["website", "site", "網站"]) else "Artifact"
        title = "Generate website workflow" if target == "Website" else "Create governed artifact workflow"
        return {
            "classification": "project_work",
            "title": title,
            "route": "prompt-to-WorkItem",
            "target": target,
            "workflow_plan": [
                {"id": "prd", "label": "PRD", "state": "ready", "depends_on": []},
                {"id": "td", "label": "TD", "state": "not-started", "depends_on": ["prd"]},
                {"id": "website", "label": "Website", "state": "not-started", "depends_on": ["td"]},
            ],
            "qc_status": "pass",
            "qc_checks": [
                {
                    "id": "intent",
                    "label": "Intent is project work",
                    "status": "pass",
                    "summary": "Prompt requests a governed artifact workflow.",
                },
                {
                    "id": "artifact_graph",
                    "label": "Artifact graph",
                    "status": "pass",
                    "summary": "PRD, TD, and Website dependencies are present.",
                },
            ],
        }

    def create_artifact(self, workitem: dict, kind: str) -> dict:
        """Create deterministic artifact content for an accepted WorkItem."""
        upper_kind = kind.upper()
        artifact_id = f"{workitem['id']}-{kind}-v1"
        return {
            "artifact": {
                "id": artifact_id,
                "workitem_id": workitem["id"],
                "label": f"{workitem['title']} {upper_kind}",
                "kind": kind,
                "status": "Draft",
                "summary": f"Deterministic {upper_kind} artifact created from the accepted WorkItem.",
                "qc_status": "pending",
                "qc_checks": [
                    {
                        "id": f"{kind}_review",
                        "label": f"{upper_kind} owner review",
                        "status": "pending",
                        "summary": f"{upper_kind} artifact is waiting for review.",
                    }
                ],
                "versions": [{"id": artifact_id, "version": 1, "status": "current"}],
            },
            "qc_result": {
                "status": "pending",
                "checks": [
                    {
                        "id": f"{kind}_review",
                        "label": f"{upper_kind} owner review",
                        "status": "pending",
                        "summary": f"{upper_kind} artifact is waiting for review.",
                    }
                ],
            },
        }

    def agent_team_artifact(self, prompt: str, roles: list[str]) -> dict:
        """Return the agent-team artifact contract without calling an LLM."""
        return {
            "status": "NotImplemented",
            "issue": "#1545 T4",
            "roles": list(roles),
            "prompt": prompt,
            "requirements_summary": "",
            "app_spec_changes": [],
            "implementation": "",
            "tests": [],
            "release_package": {},
            "review_tickets": [],
            "schema_version": "cue.agent-team-artifact.v0",
        }


def slug(value: str) -> str:
    """Stable slug helper shared by fixture and future repository providers."""
    normalized = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return normalized or "workitem"


def artifact_to_json(artifact: dict) -> str:
    """Serialize deterministic artifact output for compatibility adapters."""
    return json.dumps(artifact)
