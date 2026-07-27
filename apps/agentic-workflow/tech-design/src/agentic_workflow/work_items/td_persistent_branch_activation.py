"""Dirty-worktree-safe TD activation on persistent project branches.

@spec #2776
"""

from __future__ import annotations

from dataclasses import dataclass


__aw_artifact_id__ = "artifact:td-cb-lifecycle-automation/persistent-branch-activation"
__aw_work_item__ = "2776"


@dataclass(frozen=True)
class TdActivationDecision:
    """Branch-aware precondition for starting one TD lifecycle."""

    current_branch: str
    branch_switch_required: bool
    require_clean_tree: bool
    reject_preexisting_staged_paths: bool


def td_activation_decision(current_branch: str) -> TdActivationDecision:
    """Keep persistent project branches in place and protect real switches."""

    branch_switch_required = current_branch == "main"
    return TdActivationDecision(
        current_branch=current_branch,
        branch_switch_required=branch_switch_required,
        require_clean_tree=branch_switch_required,
        reject_preexisting_staged_paths=not branch_switch_required,
    )


def lifecycle_owned_paths(issue_path: str, td_source_path: str) -> tuple[str, str]:
    """Limit lifecycle commits to the hydrated WI and generated TD source."""

    return issue_path, td_source_path
