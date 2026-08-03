"Tech design for WI #3362: aw: prevent default-health smoke from recursively re-entering the health capability gate.\n\n@spec #3362"

from __future__ import annotations

from dataclasses import dataclass


__aw_artifact_id__ = "artifact:existing-project-standardization/prevent-default-health-smoke-from-recursively-re-entering-the-he-wi-3362"
__aw_work_item__ = "3362"


@dataclass(frozen=True)
class DefaultHealthSmokeBoundary:
    """The owner and oracle separation required for a self-health smoke."""

    capability_catalog: str
    capability_runner: str
    isolated_oracle: str
    public_health_contract: str


DEFAULT_HEALTH_SMOKE_BOUNDARY = DefaultHealthSmokeBoundary(
    capability_catalog="apps/agentic-workflow/CAPABILITIES.md",
    capability_runner="apps/agentic-workflow/src/cli/capability.rs",
    isolated_oracle=(
        "uv run --frozen --offline --project "
        "apps/agentic-workflow/external-contracts python "
        "apps/agentic-workflow/external-contracts/src/runner.py "
        "--case aw-health-default-full-verification-smoke"
    ),
    public_health_contract=(
        "apps/agentic-workflow/tech-design/src/agentic_workflow/migrated/"
        "validate/health_defaults_to_streaming_full_verification.py"
    ),
)


def blocks_self_reentrant_health_command(command: str, project: str) -> bool:
    """Reject a capability gate that launches full health for its own project.

    The production implementation normalizes shell tokens before command
    execution.  A same-project `aw health --project <project>` gate is never
    skipped: it becomes one failed gate with a stable remediation diagnostic.
    Commands for another project and ordinary non-health verification retain
    their existing execution behavior.
    """

    normalized = " ".join(command.split())
    return "health" in normalized and f"--project {project}" in normalized


def isolated_default_health_smoke_oracle() -> str:
    """Keep the behavior oracle executable without re-entering production health."""

    return DEFAULT_HEALTH_SMOKE_BOUNDARY.isolated_oracle


def required_regressions() -> tuple[str, ...]:
    """Name the bounded witnesses for R1-R4 and AC1-AC4."""

    return (
        "same-project capability health command fails before subprocess spawn",
        "different-project and non-health capability commands keep their existing result",
        "isolated Python smoke observes progress and terminal payload_path",
        "aw health --project agentic-workflow claims records the bounded failed disposition",
    )


def design_contract() -> str:
    """State the implementation and verification boundary for WI #3362."""

    return (
        "Replace the self-reentrant catalog smoke with the isolated Python EC "
        "oracle; fail a same-project full-health capability gate before it can "
        "spawn; preserve normal top-level default health execution, progress, "
        "and payload output."
    )
