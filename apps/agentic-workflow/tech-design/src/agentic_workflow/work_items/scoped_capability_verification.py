"""Dependency-closed capability goal verification.

@spec #2770
"""

from __future__ import annotations

from dataclasses import dataclass


__aw_artifact_id__ = "artifact:capability-control-plane/scoped-capability-verification"
__aw_work_item__ = "2770"


@dataclass(frozen=True)
class CapabilityNode:
    """The minimum catalog input needed to resolve a verification scope."""

    capability_id: str
    dependencies: tuple[str, ...] = ()


def dependency_closed_scope(
    catalog: tuple[CapabilityNode, ...],
    requested_capability_id: str,
) -> tuple[str, ...]:
    """Return the requested capability plus every transitive dependency.

    Unknown references and cycles fail before any verification command can run.
    """

    by_id = {node.capability_id: node for node in catalog}
    if requested_capability_id not in by_id:
        raise ValueError(f"capability `{requested_capability_id}` is not declared")

    scope: set[str] = set()
    visiting: list[str] = []

    def visit(capability_id: str) -> None:
        if capability_id in visiting:
            cycle = visiting[visiting.index(capability_id) :] + [capability_id]
            raise ValueError(
                "capability dependency cycle detected: " + " -> ".join(cycle)
            )
        if capability_id in scope:
            return
        node = by_id.get(capability_id)
        if node is None:
            raise ValueError(f"dependency `{capability_id}` is not declared")
        visiting.append(capability_id)
        scope.add(capability_id)
        for dependency in node.dependencies:
            visit(dependency)
        visiting.pop()

    visit(requested_capability_id)
    return tuple(sorted(scope))


def scoped_check_command(project: str, capability_id: str) -> str:
    """Render the scope-preserving command used by progress and blockers."""

    return (
        f"aw capability check --project {project} --verify "
        f"--capability {capability_id}"
    )


def workspace_test_gates_required(capability_id: str | None) -> bool:
    """Project-wide workspace test commands belong only to the project root."""

    return capability_id is None
