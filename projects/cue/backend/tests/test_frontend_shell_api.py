"""Contract tests for the Cue React-on-Jet frontend shell manifest."""

from __future__ import annotations

import asyncio
from pathlib import Path

from test_workstream_api import _load_main


def test_frontend_shell_manifest_tracks_react_tsx_sites_and_jet_substrate() -> None:
    main = _load_main()
    handler = main.app._handlers[("GET", "/api/admin/frontend-shell")]

    payload = asyncio.run(handler())

    assert payload["ok"] is True
    manifest = payload["data"]
    assert manifest["schema_version"] == "cue.react-on-jet-shell.v0"
    assert manifest["substrate"]["owner"] == "jet"
    assert manifest["substrate"]["target_package_manager"] == "jet"
    assert manifest["substrate"]["current_bridge"] == "vite"
    assert manifest["substrate"]["cue_workaround_policy"] == "narrow_and_removable"

    sites = {site["name"]: site for site in manifest["sites"]}
    assert sites["artifact_studio"]["audience"] == "project_owner"
    assert sites["admin"]["audience"] == "platform_operator"
    cue_root = Path(__file__).resolve().parents[2]
    for site in sites.values():
        assert site["ui_runtime"] == "react_tsx"
        assert (cue_root.parents[1] / site["path"] / site["entrypoint"]).exists()


def test_frontend_shell_manifest_exposes_validation_commands() -> None:
    main = _load_main()
    handler = main.app._handlers[("GET", "/api/admin/frontend-shell")]

    payload = asyncio.run(handler())
    validation = payload["data"]["validation"]

    assert validation["artifact_studio"]["typecheck"] == "npm run typecheck"
    assert validation["artifact_studio"]["build"] == "npm run build"
    assert validation["artifact_studio"]["e2e"] == "npm run test:e2e"
    assert validation["admin"]["typecheck"] == "npm run typecheck"
    assert validation["admin"]["build"] == "npm run build"
    assert "project:jet issue" in validation["jet_blocker_policy"]
