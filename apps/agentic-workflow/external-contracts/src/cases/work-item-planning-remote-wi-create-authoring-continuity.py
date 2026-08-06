"""Black-box contract for remote wi-create authoring continuity (#3303)."""

from __future__ import annotations

import json
import re
import stat
import sys
import tempfile
import threading
from collections.abc import Iterator
from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import final_json, run_aw


CASE_ID = "work-item-planning-remote-wi-create-authoring-continuity"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "remote-wi-create-authoring-continuity"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-remote-wi-create-authoring-continuity"
)
ASSERTIONS = (
    "wi create against a configured remote backend mirrors the tracker-returned numeric issue id into local workspace lifecycle state as the canonical slug, not a title-derived local slug -- proving remote creation enters the same linear authoring loop under the tracker's own identity",
    "the accepted fill-section payload is pushed to the configured remote tracker as a labeled PATCH call strictly inside the fill-apply step and strictly before wi validate is ever invoked, with no such push observable beforehand -- proving the accepted fill is projected back to the tracker before validation",
    "wi validate on the remotely mirrored work item terminates in one hop with a done envelope reporting passed=true and the promoted open state, exactly as local-only authoring does -- proving validation is unchanged by the remote round trip",
)

_MARKER = "RemoteContinuityMarker9f3"
_REPO = "fixture/continuity"
_ISSUE_NUMBER = 9001

_EPIC_BODY = (
    "## Requirements\n\n- R1: Demonstrate remote continuity.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw wi validate <slug>` | validate reports passed. | - |\n"
)


def _filled_epic_body() -> str:
    return (
        "## Problem\n\nDemonstrate remote authoring continuity end to end.\n\n"
        "## Capability Alignment\n\n"
        "Capability: Work-item planning\n"
        "Capability Gap: none, this fixture only drives the existing remote-create pipeline\n"
        "Progress Evidence: the public wi validate envelope is the evidence\n\n"
        f"## Requirements\n\n- R1: Demonstrate remote continuity. Marker={_MARKER}.\n\n"
        "## Scope\n\n### In Scope\n- trace the remote create/fill/validate round trip.\n\n"
        "### Out of Scope\n- Rework unrelated lifecycle stages.\n\n"
        "## Acceptance Criteria\n\n- AC1: validate passes against the tracker-mirrored slug.\n\n"
        "## Verification Inventory\n\n"
        "| Requirement | Gate | Oracle | Depends On |\n"
        "|-------------|------|--------|------------|\n"
        "| R1 | `aw wi validate <slug>` | validate reports passed. | - |\n\n"
        "## Reference Context\n\n### Related Specs\n"
        "| Spec | Relevance |\n|------|-----------|\n"
        "| complete-platform.md | describes the environment |\n\n"
        "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
        "|---------|--------|---------------|\n"
        "| remote-continuity-trace | update | complete-platform.md |\n"
    )


class _TrackerState:
    def __init__(self) -> None:
        self.calls: list[str] = []
        self.body = "(fixture-created body)"
        self.title = "remote continuity epic"


def _issue_payload(state: "_TrackerState") -> dict[str, Any]:
    return {
        "number": _ISSUE_NUMBER,
        "title": state.title,
        "state": "OPEN",
        "labels": [{"name": "type:epic"}],
        "author": {"login": "fixture"},
        "createdAt": "2026-07-30T00:00:00Z",
        "updatedAt": "2026-07-30T00:00:00Z",
        "url": f"https://example.invalid/{_REPO}/issues/{_ISSUE_NUMBER}",
        "body": state.body,
    }


def _tracker_handler(state: _TrackerState) -> type[BaseHTTPRequestHandler]:
    class Handler(BaseHTTPRequestHandler):
        def do_POST(self) -> None:  # noqa: N802 (stdlib override)
            length = int(self.headers.get("Content-Length", "0"))
            body = self.rfile.read(length).decode(errors="replace")
            state.calls.append(body)

            if f"POST repos/{_REPO}/issues -f" in body:
                # `gh api -X POST repos/<repo>/issues ...` -- issue create.
                self._reply(200, json.dumps(_issue_payload(state)))
                return
            if f"PATCH repos/{_REPO}/issues/{_ISSUE_NUMBER}" in body:
                # `gh api -X PATCH repos/<repo>/issues/<n> ...` -- edit.
                if _MARKER in body:
                    state.body = _filled_epic_body()
                self._reply(200, json.dumps(_issue_payload(state)))
                return
            if f"view {_ISSUE_NUMBER} " in body or body.strip().endswith(
                f"view {_ISSUE_NUMBER}"
            ):
                self._reply(200, json.dumps(_issue_payload(state)))
                return
            if body.startswith("label create "):
                self._reply(200, "{}")
                return
            if "/comments" in body:
                self._reply(200, "{}")
                return
            self._reply(400, f"unexpected fixture gh invocation: {body}")

        def _reply(self, status: int, payload: str) -> None:
            data = payload.encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def log_message(self, _format: str, *_args: Any) -> None:
            return

    return Handler


@contextmanager
def _remote_project() -> Iterator[tuple[Path, _TrackerState, dict[str, str]]]:
    with tempfile.TemporaryDirectory(prefix="aw-python-ec-remote-continuity-") as raw_root:
        root = Path(raw_root)
        (root / "aw.toml").write_text(
            "[agentic_workflow.workspace]\n"
            'mode = "in_place"\n\n'
            "[agentic_workflow.issue_platform]\n"
            'type = "github"\n'
            f'repo = "{_REPO}"\n\n'
            "[[projects]]\n"
            'name = "demo"\n'
            'label = "app:demo"\n'
            'path = "."\n'
            'tech_design_path = "tech-design"\n\n'
            "[[projects.workspaces]]\n"
            'name = "demo"\n'
            'paths = ["**"]\n'
            'target = "rust"\n',
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
            "#!/bin/sh\n"
            'exec /usr/bin/curl --silent --show-error --fail -X POST --data "$*" '
            '"$AW_GH_FIXTURE_URL/gh"\n',
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


def _workspace_slug(root: Path) -> str:
    resolved = str(root.resolve())
    collapsed = re.sub(r"[^a-zA-Z0-9]+", "-", resolved)
    return collapsed.strip("-").lower()


def _payload_path(root: Path, slug: str) -> Path:
    return (
        Path("/tmp/aw/workspaces")
        / _workspace_slug(root)
        / "payloads"
        / "wi"
        / slug
        / "body.md"
    )


def verify() -> list[str]:
    with _remote_project() as (root, state, env):
        created = final_json(
            run_aw(
                root,
                "wi",
                "create",
                "--title",
                "Remote continuity epic",
                "--type",
                "epic",
                "--project",
                "demo",
                "--body",
                _EPIC_BODY,
                env_overrides=env,
            )
        )
        slug = created["slug"]

        # Cluster 1: the numeric tracker id is mirrored as the canonical
        # slug, not a title-derived local slug.
        assert slug == str(_ISSUE_NUMBER), created
        assert re.fullmatch(r"\d+", slug), created
        assert not any(_MARKER in call for call in state.calls), state.calls

        payload_path = _payload_path(root, slug)
        payload_path.parent.mkdir(parents=True, exist_ok=True)
        payload_path.write_text(_filled_epic_body(), encoding="utf-8")

        calls_before_fill = len(state.calls)
        run_aw(
            root,
            "wi",
            "fill-section",
            "--slug",
            slug,
            "--section",
            "all",
            "--apply",
            env_overrides=env,
        )
        fill_calls = state.calls[calls_before_fill:]

        # Cluster 2: the accepted fill is pushed to the tracker as a PATCH
        # bearing the accepted content, strictly inside the fill-apply step
        # and strictly before validate is ever invoked.
        marker_patch_calls = [
            call
            for call in fill_calls
            if f"PATCH repos/{_REPO}/issues/{_ISSUE_NUMBER}" in call and _MARKER in call
        ]
        assert marker_patch_calls, fill_calls
        assert not any(_MARKER in call for call in state.calls[:calls_before_fill]), state.calls

        validated = final_json(run_aw(root, "wi", "validate", slug, env_overrides=env))

        # Cluster 3: validate still terminates in one hop with a done
        # envelope, exactly as local-only authoring does.
        assert validated["status"] == "done", validated
        assert validated["passed"] is True, validated
        assert validated["new_state"] == "open", validated
        assert validated["completion"]["workflow_complete"] is True, validated

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
