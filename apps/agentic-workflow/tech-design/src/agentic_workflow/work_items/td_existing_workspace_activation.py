"""Branch-aware activation for existing TD workspaces.

@spec #2779
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum


__aw_artifact_id__ = "artifact:td-cb-lifecycle-automation/existing-workspace-activation"
__aw_work_item__ = "2779"


class ExistingWorkspaceAction(StrEnum):
    """The only branch actions an existing-workspace verb may take."""

    STAY = "stay"
    SWITCH = "switch"
    MISSING = "missing"


@dataclass(frozen=True)
class ExistingWorkspaceActivation:
    """Preflight decision shared by TD and CB existing-workspace verbs."""

    action: ExistingWorkspaceAction
    require_clean_tree: bool
    reject_preexisting_staged_paths: bool


def existing_workspace_activation(
    current_branch: str,
    td_branch_exists: bool,
) -> ExistingWorkspaceActivation:
    """Keep persistent branches in place; protect only a real main switch."""

    if current_branch != "main":
        return ExistingWorkspaceActivation(
            action=ExistingWorkspaceAction.STAY,
            require_clean_tree=False,
            reject_preexisting_staged_paths=True,
        )
    if td_branch_exists:
        return ExistingWorkspaceActivation(
            action=ExistingWorkspaceAction.SWITCH,
            require_clean_tree=True,
            reject_preexisting_staged_paths=True,
        )
    return ExistingWorkspaceActivation(
        action=ExistingWorkspaceAction.MISSING,
        require_clean_tree=False,
        reject_preexisting_staged_paths=True,
    )
