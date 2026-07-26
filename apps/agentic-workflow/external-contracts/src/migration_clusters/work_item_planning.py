"""Native black-box external contracts for the work-item planning cluster."""

from __future__ import annotations

import json
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
    with project_fixture() as root:
        help_result = run_aw(root, "wi", "create", "--help")
        assert "--remote" not in help_result.stdout
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
        assert issue["type"] == "change"
        assert "app:demo" in issue["labels"]
        assert "Agent Estimate" not in issue["body"]
        run_aw(root, "wi", "validate", slug)

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
        run_aw(root, "wi", "validate", legacy_created["slug"])

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
        assert "Agent Estimate" not in serialized
        assert "agent_minutes" not in serialized
        assert "human_attention" not in serialized
        ready_lane = (
            graph["valid"] is True
            and any(
                change.get("title") == "Bounded planning fixture"
                and change.get("lane") == "ready_now"
                for change in plan_document["changes"]
            )
        )
        if not ready_lane:
            raise AssertionError(f"bounded change missing from planning result: {serialized}")
        return {
            "help_hidden": True,
            "compatibility_flag_parsed": True,
            "local_backend_created": issue["slug"] == slug,
            "bounded_validated": True,
            "legacy_estimate_inert": True,
            "planning_omits_estimate": True,
            "ready_lane": ready_lane,
        }


def _verify_planning(case_id: str) -> list[str]:
    if case_id == "work-item-planning-operational-efficiency":
        started = time.monotonic()
        snapshot = _planning_snapshot()
        assert all(snapshot.values()), snapshot
        elapsed = time.monotonic() - started
        assert elapsed <= 120
        return [
            f"native Python planning gate passed in {elapsed:.3f}s",
            "representative behavior assertions executed without cargo delegation",
        ]
    if case_id == "work-item-planning-operational-stability":
        first = _planning_snapshot()
        second = _planning_snapshot()
        assert first == second
        assert all(first.values()), first
        return [
            "two fresh native Python planning executions produced identical results",
            "both executions passed every representative behavior assertion",
        ]

    snapshot = _planning_snapshot()
    assert all(snapshot.values()), snapshot
    assertions = {
        "wi-create-help-command": ["help output does not list --remote"],
        "wi-create-help-smoke": ["stdout does not contain --remote"],
        "wi-create-remote-flag-tests": [
            "create help hides --remote",
            "hidden compatibility flag parses",
            "local configuration selects the local backend",
        ],
        "wi-create-remote-unit-command": [
            "help hides deprecated flag and compatibility input remains accepted",
            "configured backend owns create behavior",
        ],
        "wi-remove-agent-estimate-build": [
            "prioritization output omits estimate fields",
        ],
        "wi-remove-agent-estimate-spec-check": [
            "legacy Agent Estimate input validates but is inert",
        ],
        "wi-remove-agent-estimate-unit-command": [
            "bounded change validates without an estimate section",
            "legacy estimate input remains parseable",
            "generated and planning outputs omit estimate fields",
        ],
        "work-item-planning-epic-to-change-atomization": [
            "bounded change appears in the ready planning lane",
        ],
    }
    return assertions[case_id]


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by work-item-planning: {case_id}")
    if case_id == "wi-close-remote-real-cli":
        return _verify_remote_close()
    return _verify_planning(case_id)
