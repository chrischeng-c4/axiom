"""Executable design for the Guard application package boundary."""

from dataclasses import dataclass

__aw_artifact_id__ = "artifact:guard/design-project-layout"


@dataclass(frozen=True)
class GuardProjectLayout:
    library_crate: str
    cli_crate: str
    capability_contract: str
    external_contract_root: str
    tech_design_root: str


def canonical_project_layout() -> GuardProjectLayout:
    return GuardProjectLayout(
        library_crate="apps/guard",
        cli_crate="apps/guard/guard-cli",
        capability_contract="apps/guard/CAPABILITIES.md",
        external_contract_root="apps/guard/external-contracts",
        tech_design_root="apps/guard/tech-design",
    )
