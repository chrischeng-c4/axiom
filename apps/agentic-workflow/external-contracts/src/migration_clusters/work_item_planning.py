"""Native black-box external contracts for the work-item planning cluster."""

from __future__ import annotations

import json
import shlex
import stat
import tempfile
import threading
import time
from collections.abc import Iterator
from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

from wi_contract_fixture import create, final_json, project_fixture, run_aw, show


CASE_IDS = {
    "wi-close-remote-real-cli",
    "wi-create-help-command",
    "wi-create-help-smoke",
    "wi-create-remote-flag-tests",
    "wi-create-remote-unit-command",
    "wi-remove-agent-estimate-build",
    "wi-remove-agent-estimate-spec-check",
    "wi-remove-agent-estimate-unit-command",
    "wi-typed-epic-owner",
    "wi-typed-priority-label",
    "work-item-planning-epic-to-change-atomization",
    "work-item-planning-operational-efficiency",
    "work-item-planning-operational-stability",
}

BOUNDED_BODY = """\
## Problem

The fixture needs a bounded planning contract that can be observed end to end.

## Capability Alignment

Capability: Work-item planning
Capability Gap: bounded changes need deterministic readiness
Progress Evidence: the Python EC observes the real CLI result

## Scope

### In Scope

- verify one bounded planning change

### Out of Scope

- tracker mutation outside the fixture

## Acceptance Criteria

- AC1: the bounded change validates and enters the ready lane

## Reference Context

### Related Specs

| Spec | Relevance |
|------|-----------|
| work-item-planning.md | high |

### Spec Plan

| Spec ID | Action | Main Spec Ref |
|---------|--------|---------------|
| work-item-planning | modify | work-item-planning.md |
"""


class _TrackerState:
    def __init__(self) -> None:
        self.calls: list[str] = []
        self.closed = False


def _tracker_handler(state: _TrackerState) -> type[BaseHTTPRequestHandler]:
    class Handler(BaseHTTPRequestHandler):
        def do_POST(self) -> None:
            length = int(self.headers.get("Content-Length", "0"))
            body = self.rfile.read(length).decode()
            state.calls.append(body)
            if " view 404 " in body:
                self.send_error(404, "fixture issue not found")
                return
            if " view 42 " in body:
                payload = {
                    "number": 42,
                    "title": "remote fixture",
                    "state": "CLOSED" if state.closed else "OPEN",
                    "labels": [{"name": "type:change"}],
                    "author": {"login": "fixture"},
                    "createdAt": "2026-07-13T00:00:00Z",
                    "updatedAt": "2026-07-13T00:00:00Z",
                    "url": "https://example.invalid/fixture/issues/42",
                    "body": "## Scope\n\nremote-only fixture",
                }
                self._reply(200, json.dumps(payload))
                return
            if " state=closed" in body:
                state.closed = True
                self._reply(200, "{}")
                return
            if "/comments" in body:
                self._reply(200, "{}")
                return
            self._reply(400, f"unexpected fixture gh invocation: {body}")

        def _reply(self, status: int, body: str) -> None:
            encoded = body.encode()
            self.send_response(status)
            self.send_header("Content-Length", str(len(encoded)))
            self.end_headers()
            self.wfile.write(encoded)

        def log_message(self, _format: str, *_args: Any) -> None:
            return

    return Handler


@contextmanager
def _remote_project() -> Iterator[tuple[Path, _TrackerState, dict[str, str]]]:
    with tempfile.TemporaryDirectory(prefix="aw-python-ec-remote-") as raw_root:
        root = Path(raw_root)
        (root / "aw.toml").write_text(
            """\
[agentic_workflow.issue_platform]
type = "github"
repo = "fixture/configured"
""",
            encoding="utf-8",
        )
        state = _TrackerState()
        server = ThreadingHTTPServer(("127.0.0.1", 0), _tracker_handler(state))
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()

        home = root / "home"
        gh = home / ".rustup/toolchains/stable-aarch64-apple-darwin/bin/gh"
        gh.parent.mkdir(parents=True)
        gh.write_text(
            """#!/bin/sh
exec /usr/bin/curl --silent --show-error --fail -X POST --data "$*" "$AW_GH_FIXTURE_URL/gh"
""",
            encoding="utf-8",
        )
        gh.chmod(gh.stat().st_mode | stat.S_IXUSR)
        env = {
            "HOME": str(home),
            "GH_TOKEN": "fixture-token",
            "AW_GH_FIXTURE_URL": f"http://127.0.0.1:{server.server_port}",
        }
        try:
            yield root, state, env
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=5)


def _verify_remote_close() -> list[str]:
    with _remote_project() as (root, state, env):
        repo = "fixture/explicit-1551"
        show_remote = run_aw(
            root,
            "wi",
            "show",
            "42",
            "--repo",
            repo,
            env_overrides=env,
        )
        assert final_json(show_remote)["state"] == "open"

        for _attempt in range(2):
            close = run_aw(
                root,
                "wi",
                "close",
                "42",
                "--push",
                "--repo",
                repo,
                "--reason",
                "fixture reason",
                env_overrides=env,
            )
            assert close.stdout.strip() == "Closed 42"

        view_calls = [call for call in state.calls if " view 42 " in call]
        close_calls = [
            call
            for call in state.calls
            if "api -X PATCH repos/fixture/explicit-1551/issues/42" in call
            and "state=closed" in call
        ]
        reason_calls = [
            call
            for call in state.calls
            if "api -X POST repos/fixture/explicit-1551/issues/42/comments" in call
            and "body=fixture reason" in call
        ]
        assert len(view_calls) == 3
        assert all("--repo fixture/explicit-1551" in call for call in view_calls)
        assert len(close_calls) == 1
        assert len(reason_calls) == 1

        missing = run_aw(
            root,
            "wi",
            "close",
            "404",
            "--push",
            "--repo",
            "fixture/missing-1551",
            "--json",
            expect_success=False,
            env_overrides=env,
        )
        assert '"code":"NOT_FOUND"' in missing.stderr
        assert "github backend" in missing.stderr
        assert "repository 'fixture/missing-1551'" in missing.stderr
        assert "aw wi show 404 --repo fixture/missing-1551" in missing.stderr
        assert state.calls[-1].find(" view 404 ") >= 0

    with project_fixture() as root:
        created = create(
            root,
            "Local close fixture",
            "change",
            "--body",
            BOUNDED_BODY,
        )
        slug = created["slug"]
        run_aw(root, "wi", "close", slug)
        assert show(root, slug)["state"] == "closed"

    return [
        "numeric remote close rehydrates through the selected repository",
        "reason and close mutations occur exactly once across retry",
        "missing remote reports backend, repository, and executable recovery command",
        "local close still moves a work item from open to closed",
    ]


def _planning_snapshot() -> dict[str, Any]:
    """Observe one real planning run and return only run-derived values.

    Every field is read back out of a public `aw` result, so a caller that
    asserts on this snapshot is constrained by observed CLI behavior instead
    of by constants decided inside this helper. Nothing here raises on the
    observations themselves; the per-case callers own the assertions.
    """
    with project_fixture() as root:
        help_result = run_aw(root, "wi", "create", "--help")
        help_lists_remote = "--remote" in help_result.stdout
        epic = create(
            root,
            "Planning fixture epic",
            "epic",
            "--priority",
            "p1",
        )
        run_aw(root, "wi", "update", epic["slug"], "--state", "open")

        created = create(
            root,
            "Bounded planning fixture",
            "change",
            "--priority",
            "p1",
            "--remote",
            "--body",
            BOUNDED_BODY,
        )
        slug = created["slug"]
        run_aw(
            root,
            "wi",
            "update",
            slug,
            "--state",
            "open",
            "--add-label",
            f"epic:{epic['slug']}",
        )
        issue = show(root, slug)
        bounded_validate = run_aw(root, "wi", "validate", slug, expect_success=None)

        legacy = BOUNDED_BODY + """\

## Agent Estimate

agent_minutes: 45
confidence: medium
risk: medium
human_attention: confirm
"""
        legacy_created = create(
            root,
            "Legacy estimate fixture",
            "change",
            "--priority",
            "p2",
            "--body",
            legacy,
        )
        run_aw(
            root,
            "wi",
            "update",
            legacy_created["slug"],
            "--state",
            "open",
            "--add-label",
            f"epic:{epic['slug']}",
        )
        legacy_slug = legacy_created["slug"]
        legacy_issue = show(root, legacy_slug)
        legacy_validate = run_aw(
            root, "wi", "validate", legacy_slug, expect_success=None
        )

        planned = final_json(
            run_aw(root, "wi", "prioritize", "--project", "demo", "--json")
        )
        graph = final_json(run_aw(root, "wi", "graph", "--project", "demo", "--json"))
        plan_path = Path(planned["plan"]["path"])
        plan_document = json.loads(plan_path.read_text(encoding="utf-8"))
        serialized = json.dumps(
            {"envelope": planned, "graph": graph, "plan": plan_document},
            sort_keys=True,
        )
        estimate_tokens = tuple(
            token
            for token in (
                "Agent Estimate",
                "agent_minutes",
                "human_attention",
                "confidence",
                "risk",
            )
            if token in serialized
        )
        return {
            "help_lists_remote": help_lists_remote,
            "epic_slug": str(epic["slug"]),
            "bounded_requested_slug": str(slug),
            "bounded_shown_slug": str(issue["slug"]),
            "bounded_type": str(issue["type"]),
            "bounded_labels": tuple(sorted(str(item) for item in issue["labels"])),
            "bounded_body_estimate_section": "## Agent Estimate" in str(issue["body"]),
            "bounded_validate_returncode": bounded_validate.returncode,
            "legacy_slug": str(legacy_slug),
            "legacy_body_estimate_section": "## Agent Estimate"
            in str(legacy_issue["body"]),
            "legacy_validate_returncode": legacy_validate.returncode,
            "graph_valid": graph["valid"],
            "planning_estimate_tokens": estimate_tokens,
            "planning_lanes": tuple(
                sorted(
                    (str(change.get("title")), str(change.get("lane")))
                    for change in plan_document["changes"]
                )
            ),
        }


def _planning_expectations(snapshot: dict[str, Any]) -> None:
    """Assert the whole observed planning contract, field by observed field."""
    assert snapshot["help_lists_remote"] is False, snapshot
    assert snapshot["bounded_shown_slug"] == snapshot["bounded_requested_slug"], snapshot
    assert snapshot["bounded_type"] == "change", snapshot
    assert "app:demo" in snapshot["bounded_labels"], snapshot
    assert f"epic:{snapshot['epic_slug']}" in snapshot["bounded_labels"], snapshot
    assert snapshot["bounded_body_estimate_section"] is False, snapshot
    assert snapshot["bounded_validate_returncode"] == 0, snapshot
    assert snapshot["legacy_slug"] == "change-legacy-estimate-fixture", snapshot
    assert snapshot["legacy_body_estimate_section"] is True, snapshot
    assert snapshot["legacy_validate_returncode"] == 0, snapshot
    assert snapshot["graph_valid"] is True, snapshot
    assert snapshot["planning_estimate_tokens"] == (), snapshot
    # Both lanes, not just the bounded one: the legacy work item carries the
    # inert Agent Estimate section, so its own ready-lane placement is the
    # observable that would reveal that section creating a readiness
    # requirement. Asserting only the bounded lane left that regression
    # undetectable by every case in this cluster.
    assert ("Bounded planning fixture", "ready_now") in snapshot[
        "planning_lanes"
    ], snapshot
    assert ("Legacy estimate fixture", "ready_now") in snapshot[
        "planning_lanes"
    ], snapshot


def _verify_planning(case_id: str) -> list[str]:
    if case_id == "wi-typed-priority-label":
        with project_fixture() as root:
            help_result = run_aw(root, "wi", "create", "--help")
            assert "priority:<value>" in help_result.stdout
            assert "priority::<value>" not in help_result.stdout

            created = create(
                root,
                "Typed priority change",
                "change",
                "--priority",
                "p2",
                "--body",
                BOUNDED_BODY,
            )
            issue = show(root, created["slug"])
            assert "priority:p2" in issue["labels"]
            assert "priority::p2" not in issue["labels"]
        return [
            "create help documents the canonical single-colon priority label",
            "typed priority emits priority:p2",
            "typed priority never emits priority::p2",
        ]

    if case_id == "wi-typed-epic-owner":
        with project_fixture() as root:
            help_result = run_aw(root, "wi", "create", "--help")
            assert "--epic <ID>" in help_result.stdout

            epic = create(root, "Typed owner epic", "epic", "--priority", "p1")
            run_aw(root, "wi", "update", epic["slug"], "--state", "open")
            owned = create(
                root,
                "Typed owner change",
                "change",
                "--epic",
                epic["slug"],
                "--body",
                BOUNDED_BODY,
            )
            run_aw(root, "wi", "update", owned["slug"], "--state", "open")
            owned_issue = show(root, owned["slug"])
            assert f"epic:{epic['slug']}" in owned_issue["labels"]

            body_owned = create(
                root,
                "Body compatibility owner change",
                "change",
                "--body",
                BOUNDED_BODY + f"\nParent epic: #{epic['slug']}.\n",
            )
            run_aw(root, "wi", "update", body_owned["slug"], "--state", "open")
            graph = final_json(
                run_aw(root, "wi", "graph", "--project", "demo", "--json")
            )
            assert graph["valid"] is True
            assert any(
                change["id"] == owned["slug"]
                and change["parent"] == epic["slug"]
                for change in graph["changes"]
            )
            assert any(
                change["id"] == body_owned["slug"]
                and change["parent"] == epic["slug"]
                for change in graph["changes"]
            )

            unowned = create(
                root,
                "Unowned change",
                "change",
                "--body",
                BOUNDED_BODY,
            )
            ownership = unowned["invoke"]["args"]["ownership"]
            assert ownership["status"] == "unowned"
            command_template = ownership["remediation_command_template"]
            assert command_template == (
                f"aw wi update {unowned['slug']} --epic <epic-id> --push"
            )
            resolved_command = command_template.replace("<epic-id>", epic["slug"])
            resolved_args = shlex.split(resolved_command)
            assert resolved_args[0] == "aw"
            run_aw(root, *resolved_args[1:])
            assert f"epic:{epic['slug']}" in show(root, unowned["slug"])["labels"]
            updated_graph = final_json(
                run_aw(root, "wi", "graph", "--project", "demo", "--json")
            )
            assert any(
                change["id"] == unowned["slug"]
                and change["parent"] == epic["slug"]
                for change in updated_graph["changes"]
            )

            missing = run_aw(
                root,
                "wi",
                "create",
                "--title",
                "Missing owner",
                "--type",
                "change",
                "--project",
                "demo",
                "--epic",
                "does-not-exist",
                "--body",
                BOUNDED_BODY,
                expect_success=False,
            )
            assert "does not resolve" in missing.stderr

            wrong_type = run_aw(
                root,
                "wi",
                "create",
                "--title",
                "Wrong owner type",
                "--type",
                "change",
                "--project",
                "demo",
                "--epic",
                unowned["slug"],
                "--body",
                BOUNDED_BODY,
                expect_success=False,
            )
            assert "not type:epic" in wrong_type.stderr

            epic_with_owner = run_aw(
                root,
                "wi",
                "create",
                "--title",
                "Epic cannot have an owner",
                "--type",
                "epic",
                "--project",
                "demo",
                "--epic",
                epic["slug"],
                expect_success=False,
            )
            assert "valid only with --type change" in epic_with_owner.stderr

            conflicting_body = run_aw(
                root,
                "wi",
                "create",
                "--title",
                "Conflicting owner",
                "--type",
                "change",
                "--project",
                "demo",
                "--epic",
                epic["slug"],
                "--body",
                BOUNDED_BODY + "\nParent: #different-owner\n",
                expect_success=False,
            )
            assert "conflicts with body parent declaration" in conflicting_body.stderr
            assert epic["slug"] in conflicting_body.stderr
            assert "different-owner" in conflicting_body.stderr

            other_project_epic = create(root, "Other project epic", "epic")
            run_aw(
                root,
                "wi",
                "update",
                other_project_epic["slug"],
                "--remove-label",
                "app:demo",
                "--add-label",
                "app:other",
            )
            cross_project = run_aw(
                root,
                "wi",
                "create",
                "--title",
                "Cross-project owner",
                "--type",
                "change",
                "--project",
                "demo",
                "--epic",
                other_project_epic["slug"],
                "--body",
                BOUNDED_BODY,
                expect_success=False,
            )
            assert "is not in the change project" in cross_project.stderr
        return [
            "create help documents --epic",
            "typed owner emits the canonical epic label",
            "graph resolves the change under its declared epic",
            "body-only parent compatibility still establishes ownership",
            "unowned create emits an exact actionable update command template",
            "resolving and executing the template assigns typed ownership",
            "invalid, cross-project, non-epic, and conflicting owners are rejected",
        ]

    if case_id == "work-item-planning-operational-efficiency":
        started = time.monotonic()
        snapshot = _planning_snapshot()
        _planning_expectations(snapshot)
        elapsed = time.monotonic() - started
        assert elapsed <= 120
        return [
            f"native Python planning gate passed in {elapsed:.3f}s",
            "representative behavior assertions executed without cargo delegation",
        ]
    if case_id == "work-item-planning-operational-stability":
        first = _planning_snapshot()
        second = _planning_snapshot()
        _planning_expectations(first)
        _planning_expectations(second)
        assert first == second, {"first": first, "second": second}
        return [
            "two fresh native Python planning executions produced identical results",
            "both executions passed every representative behavior assertion",
        ]

    snapshot = _planning_snapshot()

    if case_id == "wi-create-help-command":
        assert snapshot["help_lists_remote"] is False, snapshot
        return ["help output does not list --remote"]
    if case_id == "wi-create-help-smoke":
        assert snapshot["help_lists_remote"] is False, snapshot
        return ["stdout does not contain --remote"]
    if case_id == "wi-create-remote-flag-tests":
        # The bounded change in the snapshot is created with `--remote`, so a
        # readable slug is itself the proof that the hidden flag still parses.
        assert snapshot["help_lists_remote"] is False, snapshot
        assert snapshot["bounded_requested_slug"], snapshot
        assert snapshot["bounded_shown_slug"] == snapshot[
            "bounded_requested_slug"
        ], snapshot
        return [
            "create help hides --remote",
            "hidden compatibility flag parses",
            "local configuration selects the local backend",
        ]
    if case_id == "wi-create-remote-unit-command":
        assert snapshot["help_lists_remote"] is False, snapshot
        assert snapshot["bounded_shown_slug"] == snapshot[
            "bounded_requested_slug"
        ], snapshot
        assert snapshot["bounded_type"] == "change", snapshot
        return [
            "help hides deprecated flag and compatibility input remains accepted",
            "configured backend owns create behavior",
        ]
    if case_id == "wi-remove-agent-estimate-build":
        assert snapshot["planning_estimate_tokens"] == (), snapshot
        return ["prioritization output omits estimate fields"]
    if case_id == "wi-remove-agent-estimate-spec-check":
        # Inert means all three at once: the legacy section is accepted, it is
        # still stored verbatim, and it reaches none of the planning output.
        assert snapshot["legacy_validate_returncode"] == 0, snapshot
        assert snapshot["legacy_body_estimate_section"] is True, snapshot
        assert snapshot["planning_estimate_tokens"] == (), snapshot
        # "creates no readiness requirement" is the second half of this case's
        # declared promise, so it needs its own observable: the work item that
        # carries the legacy section still reaches the ready lane. Without this
        # the case could not fail when the section started gating readiness.
        assert ("Legacy estimate fixture", "ready_now") in snapshot[
            "planning_lanes"
        ], snapshot
        return [
            "legacy Agent Estimate input validates but is inert",
            "the work item carrying the legacy section still reaches the ready "
            "lane, so the section creates no readiness requirement",
        ]
    if case_id == "wi-remove-agent-estimate-unit-command":
        assert snapshot["bounded_validate_returncode"] == 0, snapshot
        assert snapshot["bounded_body_estimate_section"] is False, snapshot
        assert snapshot["legacy_validate_returncode"] == 0, snapshot
        assert snapshot["planning_estimate_tokens"] == (), snapshot
        return [
            "bounded change validates without an estimate section",
            "legacy estimate input remains parseable",
            "generated and planning outputs omit estimate fields",
        ]
    if case_id == "work-item-planning-epic-to-change-atomization":
        assert snapshot["graph_valid"] is True, snapshot
        assert f"epic:{snapshot['epic_slug']}" in snapshot["bounded_labels"], snapshot
        assert ("Bounded planning fixture", "ready_now") in snapshot[
            "planning_lanes"
        ], snapshot
        return ["bounded change appears in the ready planning lane"]

    raise AssertionError(f"unhandled work-item-planning case: {case_id}")


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by work-item-planning: {case_id}")
    if case_id == "wi-close-remote-real-cli":
        return _verify_remote_close()
    return _verify_planning(case_id)
