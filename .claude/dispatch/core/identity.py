from __future__ import annotations

import re

TASK_KEY_PATTERN = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$")


def task_session_policy(profile: dict) -> str:
    task = profile.get("task_contract")
    if not isinstance(task, dict):
        raise SystemExit("profile missing task_contract")
    policy = task.get("session_policy", "ticketed")
    if policy not in ("ticketed", "one-shot"):
        raise SystemExit(
            "task_contract.session_policy must be ticketed or one-shot"
        )
    return policy


def revision_origin(profile: dict) -> str:
    """The issue a one-shot round descends from, when it descends from one.

    A ticketed round's identity *is* its issue number, and that number is spent
    the moment a conversation exists against it. So a correction that needs a
    fresh dispatch rather than a continuation cannot stay ticketed: it takes a
    run id and becomes one-shot. This field is what keeps the descent on the
    record afterwards, in the profile, in the prompt, and in the sealed frozen
    task state -- an intent sentence alone would say it where nothing can
    refuse it.
    """
    task = profile.get("task_contract")
    if not isinstance(task, dict):
        raise SystemExit("profile missing task_contract")
    return str(task.get("revision_of", "") or "").strip()


def validate_task_identity(profile: dict) -> str:
    task = profile.get("task_contract")
    if not isinstance(task, dict):
        raise SystemExit("profile missing task_contract")
    policy = task_session_policy(profile)
    issue = str(task.get("issue", "")).strip()
    run_id = str(task.get("run_id", "")).strip()
    origin = revision_origin(profile)
    if policy == "ticketed":
        if not issue:
            raise SystemExit(
                "ticketed task requires task_contract.issue"
            )
        if run_id:
            raise SystemExit(
                "ticketed task must not set task_contract.run_id"
            )
        if origin:
            raise SystemExit(
                "ticketed task must not set task_contract.revision_of: a "
                "contract that both is a ticket and descends from one names "
                "two rounds, and every later verb would have to guess which"
            )
        expected = issue
    else:
        if issue:
            raise SystemExit(
                "one-shot task must not set task_contract.issue"
            )
        if not run_id:
            raise SystemExit(
                "one-shot task requires task_contract.run_id"
            )
        intent = task.get("intent")
        if not isinstance(intent, str) or not intent.strip():
            raise SystemExit(
                "one-shot task requires non-empty task_contract.intent"
            )
        if origin and not TASK_KEY_PATTERN.fullmatch(origin):
            # The same rule the identity it was copied from had to satisfy, so
            # `revise` cannot mint a descent the loader will not take back.
            raise SystemExit(
                "task_contract.revision_of must be the task identity the "
                f"revision descends from, not {origin!r}"
            )
        expected = run_id
    if not TASK_KEY_PATTERN.fullmatch(expected):
        raise SystemExit(
            "task identity must match [A-Za-z0-9][A-Za-z0-9._-]{0,127}"
        )
    return expected


def validate_task_key(profile: dict, task_key: str) -> None:
    expected = validate_task_identity(profile)
    if expected != str(task_key):
        raise SystemExit(
            f"task identity {expected} does not match requested key={task_key}"
        )
