from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass

from cli_std.application.errors import NoMatchingRelease, UnreadableCurrentVersion
from cli_std.domain.errors import DigestMismatch, MissingInnerBinary
from cli_std.domain.tool_identity import ToolInfo
from cli_std.domain.version import (
    Action,
    Version,
    decide_action,
    next_command_after_check,
    parse_version,
    select_version,
)
from cli_std.infrastructure.digest import verify_sha256


@dataclass(frozen=True)
class UpgradePlan:
    action: Action
    tag: str
    version: Version
    asset_name: str
    inner_binary_path: str


@dataclass(frozen=True)
class CheckReport:
    current: Version
    selected: Version
    action: Action
    next_command: str


PlanOutcome = UpgradePlan | NoMatchingRelease | UnreadableCurrentVersion
CheckOutcome = CheckReport | NoMatchingRelease | UnreadableCurrentVersion


def plan_upgrade(
    tool: ToolInfo, tags: Sequence[str], pin: str | None, force: bool
) -> PlanOutcome:
    current = parse_version(tool.version)
    if current is None:
        return UnreadableCurrentVersion(tool.version)
    picked = select_version(tags, tool.tag_prefix(), pin)
    if picked is None:
        return NoMatchingRelease(tool.tag_prefix(), pin)
    tag, version = picked
    return UpgradePlan(
        action=decide_action(current, version, force),
        tag=tag,
        version=version,
        asset_name=tool.asset_name(),
        inner_binary_path=tool.inner_binary_path(),
    )


def plan_check(
    tool: ToolInfo, tags: Sequence[str], pin: str | None
) -> CheckOutcome:
    current = parse_version(tool.version)
    if current is None:
        return UnreadableCurrentVersion(tool.version)
    picked = select_version(tags, tool.tag_prefix(), pin)
    if picked is None:
        return NoMatchingRelease(tool.tag_prefix(), pin)
    tag, version = picked
    return CheckReport(
        current=current,
        selected=version,
        action=decide_action(current, version, force=False),
        next_command=next_command_after_check(tool.project, current, version),
    )


def accept_asset(
    payload: bytes, expected_digest: str | None
) -> None | DigestMismatch:
    if expected_digest is None:
        return None
    return verify_sha256(payload, expected_digest)


def locate_inner_binary(
    members: Sequence[str], plan: UpgradePlan
) -> str | MissingInnerBinary:
    if plan.inner_binary_path in members:
        return plan.inner_binary_path
    return MissingInnerBinary(plan.inner_binary_path)
