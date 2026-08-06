"""Black-box contract for the chat-command removal (#1503, #3307).

Drives the real `aw` binary three ways: renders `aw --help` and confirms
`chat` is genuinely absent from the top-level command tree; invokes
`aw chat post ...` directly and confirms clap rejects it as an unrecognized
subcommand with no compatibility alias; and runs a real `aw new --force`
asset refresh against a fixture project that still has a stale
`aw-chat-listen` skill installed in both the `.claude/skills/` and
`.agents/skills/` trees, proving the installer actually deletes the retired
skill directories rather than merely omitting it from documentation.
"""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import AW_BINARY, _ensure_aw_binary, run_aw

CASE_ID = "aw-core-client-cross-checkout-chat-command-and-shared-channel-removal"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "cross-checkout-chat-command-and-shared-channel-removal"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-cross-checkout-chat-command-and-shared-channel-removal"
)
ASSERTIONS = (
    "a real `aw --help` renders the full top-level command tree with no "
    "`chat` entry, and a real `aw chat post hello` is rejected by clap as "
    "an unrecognized subcommand (nonzero exit, 'unrecognized subcommand "
    "'chat'' on stderr) rather than opening a shared channel or falling "
    "back to a compatibility alias, proving the cross-checkout transport is "
    "genuinely gone from the compiled binary and not merely undocumented",
    "a fixture project carrying a pre-existing stale `aw-chat-listen` skill "
    "directory in both the `.claude/skills/` and `.agents/skills/` trees is "
    "refreshed with a real `aw new --force` asset install, which prints "
    "'aw-chat-listen (removed)' for both trees and actually deletes both "
    "directories from disk, and the resulting skill sets do not "
    "reinstall it, proving the retired listener skill is pruned by the "
    "live installer rather than only absent from a hand-maintained list",
)


def verify() -> list[str]:
    _ensure_aw_binary()

    help_result = subprocess.run(
        [str(AW_BINARY), "--help"], capture_output=True, text=True, check=False
    )
    assert help_result.returncode == 0, help_result.stderr
    help_lines = [line.strip() for line in help_result.stdout.splitlines()]
    command_lines = [line for line in help_lines if line.startswith("chat ")]
    assert not command_lines, help_result.stdout
    assert "\n  chat" not in help_result.stdout, help_result.stdout

    chat_result = subprocess.run(
        [str(AW_BINARY), "chat", "post", "hello"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert chat_result.returncode != 0, chat_result.stdout
    assert "unrecognized subcommand 'chat'" in chat_result.stderr, chat_result.stderr
    assert "channel" not in chat_result.stdout.lower(), chat_result.stdout

    with tempfile.TemporaryDirectory(prefix="aw-ec-chat-removal-") as raw_root:
        root = Path(raw_root)
        target = root / "demo-project"
        target.mkdir()

        stale_claude = target / ".claude" / "skills" / "aw-chat-listen"
        stale_claude.mkdir(parents=True)
        (stale_claude / "SKILL.md").write_text(
            "# stale aw-chat-listen (claude)\n", encoding="utf-8"
        )
        stale_agents = target / ".agents" / "skills" / "aw-chat-listen"
        stale_agents.mkdir(parents=True)
        (stale_agents / "SKILL.md").write_text(
            "# stale aw-chat-listen (agents)\n", encoding="utf-8"
        )

        install = run_aw(root, "new", "demo-project", "--path", str(target), "-f")
        assert "aw-chat-listen (removed)" in install.stdout, install.stdout
        assert install.stdout.count("aw-chat-listen (removed)") == 2, install.stdout

        assert not stale_claude.exists(), "stale .claude/skills/aw-chat-listen survived install"
        assert not stale_agents.exists(), "stale .agents/skills/aw-chat-listen survived install"

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
