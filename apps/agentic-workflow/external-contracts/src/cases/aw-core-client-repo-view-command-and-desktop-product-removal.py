"""Black-box contract for the Repo View desktop product removal (#3306).

Drives the real `aw` binary directly: renders `aw --help` and confirms
`view` is genuinely absent from the top-level command tree; invokes
`aw view` directly and confirms clap rejects it as an unrecognized
subcommand with no compatibility alias or replacement UI silently starting;
and reads the checked-out repository tree to confirm the concrete
view-only implementation paths named in the retirement's own change list
are actually deleted, while sibling files in the same directories survive,
proving a surgical deletion rather than a documentation-only claim.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import AW_BINARY, REPOSITORY_ROOT, _ensure_aw_binary

CASE_ID = "aw-core-client-repo-view-command-and-desktop-product-removal"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "repo-view-command-and-desktop-product-removal"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-repo-view-command-and-desktop-product-removal"
)
ASSERTIONS = (
    "a real `aw --help` renders the full top-level command tree with no "
    "`view` entry, and a real `aw view` invocation is rejected by clap as "
    "an unrecognized subcommand (exit code 2, \"unrecognized subcommand "
    "'view'\" on stderr, empty stdout) rather than opening a native window, "
    "browser viewer, or any compatibility alias, proving the Repo View "
    "desktop product is genuinely gone from the compiled binary's dispatch "
    "tree and not merely undocumented",
    "the concrete view-only implementation paths named in the removal's "
    "own change list -- src/cli/view.rs, src/ui/native_view.rs, "
    "src/ui/viewer/, packages/@sdd/, pnpm-workspace.yaml, pnpm-lock.yaml -- "
    "are all genuinely absent from the checked-out repository tree, while "
    "sibling files inside the very same src/cli/ and src/ui/ directories "
    "remain present, proving a surgical deletion of the desktop-product "
    "surface rather than an untouched or never-existent subtree",
)

_REMOVED_PATHS = (
    "apps/agentic-workflow/src/cli/view.rs",
    "apps/agentic-workflow/src/ui/native_view.rs",
    "apps/agentic-workflow/src/ui/viewer",
    "apps/agentic-workflow/packages/@sdd",
    "pnpm-workspace.yaml",
    "pnpm-lock.yaml",
)
_SURVIVING_SIBLINGS = (
    "apps/agentic-workflow/src/cli/chain.rs",
    "apps/agentic-workflow/src/ui/mod.rs",
    "apps/agentic-workflow/src/ui/tables.rs",
)


def verify() -> list[str]:
    _ensure_aw_binary()

    help_result = subprocess.run(
        [str(AW_BINARY), "--help"], capture_output=True, text=True, check=False
    )
    assert help_result.returncode == 0, help_result.stderr
    command_lines = [
        line for line in help_result.stdout.splitlines() if line.strip().startswith("view ")
    ]
    assert not command_lines, help_result.stdout
    assert "\n  view" not in help_result.stdout, help_result.stdout

    view_result = subprocess.run(
        [str(AW_BINARY), "view"], capture_output=True, text=True, check=False
    )
    assert view_result.returncode == 2, (view_result.returncode, view_result.stderr)
    assert "unrecognized subcommand 'view'" in view_result.stderr, view_result.stderr
    assert view_result.stdout == "", view_result.stdout
    assert "snapshot" not in view_result.stderr.lower(), view_result.stderr

    for relative in _REMOVED_PATHS:
        candidate = REPOSITORY_ROOT / relative
        assert not candidate.exists(), f"retired path still present: {relative}"

    for relative in _SURVIVING_SIBLINGS:
        candidate = REPOSITORY_ROOT / relative
        assert candidate.is_file(), f"expected surviving sibling file missing: {relative}"

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
