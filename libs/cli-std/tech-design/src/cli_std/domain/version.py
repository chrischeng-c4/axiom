from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from enum import Enum


@dataclass(frozen=True)
class Version:
    major: int
    minor: int
    patch: int
    pre: str

    def is_stable(self) -> bool:
        return self.pre == ""

    def canonical(self) -> str:
        if self.pre == "":
            return f"{self.major}.{self.minor}.{self.patch}"
        return f"{self.major}.{self.minor}.{self.patch}-{self.pre}"


class Action(Enum):
    UP_TO_DATE = "up-to-date"
    INSTALL = "install"


def parse_version(text: str) -> Version | None:
    if not text or text != text.strip() or any(c.isspace() for c in text):
        return None
    if "-" in text:
        parts = text.split("-", 1)
        core = parts[0]
        pre = parts[1]
        if pre == "":
            return None
    else:
        core = text
        pre = ""

    core_parts = core.split(".")
    if len(core_parts) != 3:
        return None

    nums: list[int] = []
    for part in core_parts:
        if not part or not all(c in "0123456789" for c in part):
            return None
        if len(part) > 1 and part.startswith("0"):
            return None
        nums.append(int(part))

    return Version(major=nums[0], minor=nums[1], patch=nums[2], pre=pre)


def compare_versions(left: Version, right: Version) -> int:
    if left.major != right.major:
        return 1 if left.major > right.major else -1
    if left.minor != right.minor:
        return 1 if left.minor > right.minor else -1
    if left.patch != right.patch:
        return 1 if left.patch > right.patch else -1

    if left.pre == "" and right.pre == "":
        return 0
    if left.pre == "" and right.pre != "":
        return 1
    if left.pre != "" and right.pre == "":
        return -1

    if left.pre < right.pre:
        return -1
    if left.pre > right.pre:
        return 1
    return 0


def parse_tag(tag: str, prefix: str) -> Version | None:
    if not tag.startswith(prefix):
        return None
    return parse_version(tag[len(prefix) :])


def select_version(
    tags: Sequence[str], prefix: str, pin: str | None
) -> tuple[str, Version] | None:
    if pin is not None:
        want = pin[len(prefix) :] if pin.startswith(prefix) else pin
        for tag in tags:
            v = parse_tag(tag, prefix)
            if v is not None and v.canonical() == want:
                return (tag, v)
        return None

    best: tuple[str, Version] | None = None
    for tag in tags:
        v = parse_tag(tag, prefix)
        if v is None or not v.is_stable():
            continue
        if best is None or compare_versions(v, best[1]) >= 0:
            best = (tag, v)
    return best


def decide_action(
    current: Version, selected: Version, force: bool
) -> Action:
    if not force and compare_versions(selected, current) == 0:
        return Action.UP_TO_DATE
    return Action.INSTALL


def next_command_after_check(
    project: str, current: Version, selected: Version
) -> str:
    if compare_versions(selected, current) > 0:
        return f"{project} upgrade"
    return "done"
