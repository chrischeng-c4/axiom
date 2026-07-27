"""Executable design for the first core-interface projection batch.

@spec #2716
"""

from __future__ import annotations

from dataclasses import dataclass


__aw_artifact_id__ = "artifact:project-local-td-and-ec-gates/migration-batch-projection-core-interfaces-01-materialize-the-te-wi-2716"
__aw_work_item__ = "2716"


@dataclass(frozen=True)
class GeneratedProjectionBatch:
    batch_id: str
    family: str
    artifact_count: int
    materialize_command: str
    verification_command: str


def projection_core_interfaces_01() -> GeneratedProjectionBatch:
    """Bind the reviewed 50-artifact batch to its producer and verifier."""

    return GeneratedProjectionBatch(
        batch_id="projection-core-interfaces-01",
        family="projection:core/interfaces",
        artifact_count=50,
        materialize_command=(
            "python3 apps/agentic-workflow/tech-design/tools/"
            "migration_reconciliation.py materialize "
            "--batch projection-core-interfaces-01"
        ),
        verification_command=(
            "python3 apps/agentic-workflow/tech-design/tools/"
            "migration_reconciliation.py verify "
            "--batch projection-core-interfaces-01"
        ),
    )


def projection_is_terminal(
    *,
    markdown_projection_exists: bool,
    python_producer_exists: bool,
    manifest_status: str,
) -> bool:
    """A projection becomes terminal only after its Python producer exists."""

    return (
        markdown_projection_exists
        and python_producer_exists
        and manifest_status == "completed"
    )
