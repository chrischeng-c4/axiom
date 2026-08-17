from __future__ import annotations

import re
import shlex


def split_rule_tokens(pattern: str) -> list[str]:
    tokens: list[str] = []
    current: list[str] = []
    escaped = False
    for character in pattern.strip():
        if character.isspace() and not escaped:
            if current:
                tokens.append("".join(current))
                current = []
            continue
        current.append(character)
        if escaped:
            escaped = False
        elif character == "\\":
            escaped = True
    if current:
        tokens.append("".join(current))
    return tokens


def rule_matches(rule: str, command: str, prefix: str = "command") -> bool:
    match = re.fullmatch(rf"{prefix}\((.*)\)", rule)
    if not match:
        return False
    pattern = match.group(1)
    if pattern == "*":
        return True
    try:
        command_tokens = shlex.split(command)
    except ValueError:
        return False
    rule_tokens = split_rule_tokens(pattern)
    if not rule_tokens or len(command_tokens) < len(rule_tokens):
        return False
    for rule_token, command_token in zip(rule_tokens, command_tokens):
        try:
            if re.fullmatch(rule_token, command_token) is None:
                return False
        except re.error:
            if rule_token != command_token:
                return False
    return True


def command_rule_matches(rule: str, command: str) -> bool:
    return rule_matches(rule, command, "command")


def task_allowlist_families(profile: dict) -> list[str]:
    return profile.get("task_commands", {}).get("allow_prefix", [])


def task_allowlist_admits(profile: dict, command: str) -> bool:
    """Does the round's own allowlist name this command line?

    Exact entries are compared as strings, because that is what the controller
    meant by writing a whole command line. Prefix entries are compared with
    `command_rule_matches`, the same function that decides whether the command
    is permitted to run at all -- so a family entry admits here exactly what it
    admits there, and the two layers cannot drift apart.
    """
    if command in profile.get("task_commands", {}).get("allow", []):
        return True
    return any(
        command_rule_matches(f"command({entry})", command)
        for entry in task_allowlist_families(profile)
    )
