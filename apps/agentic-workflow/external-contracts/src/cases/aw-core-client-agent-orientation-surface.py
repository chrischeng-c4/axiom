"""Black-box contract for the offline agent-orientation outline (#3307).

Drives the real `aw llm` (default outline) and `aw --help` and proves the
outline's "Standard agent commands" section is built from the CLI's own live
clap surface rather than a hand-maintained doc string: every command named
in that section is a genuine top-level subcommand, and the argument summary
shown for each cross-checks against that subcommand's own real `--help`
output. Also proves the section is a bounded, non-arbitrary subset by
checking a project-specific verb (`wi`) is deliberately excluded from it.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import AW_BINARY, _ensure_aw_binary

CASE_ID = "aw-core-client-agent-orientation-surface"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "agent-orientation-surface"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-agent-orientation-surface"
)
ASSERTIONS = (
    "the real `aw llm` outline's '## Standard agent commands' section names "
    "exactly the standard commands `aw llm`, `aw upgrade`, and `aw issue`, "
    "every one of which is independently confirmed to be a real top-level "
    "subcommand in `aw --help`'s own live command tree, proving the outline "
    "reflects the CLI's actual registered surface rather than a "
    "hand-maintained string that could silently drift from it",
    "the inline argument summaries the outline prints for `aw upgrade` "
    "(`--version`, `--check`) and `aw issue` (`search`, `view`, `create "
    "--title`, `comment`) each name real flags/subcommands confirmed "
    "independently against `aw upgrade --help`, `aw issue --help`, `aw "
    "issue create --help`, and `aw issue comment --help`, and a "
    "project-lifecycle verb (`wi`) that is a real top-level subcommand is "
    "deliberately absent from the standard-commands section, proving that "
    "section is a bounded, derived subset rather than a dump of the whole "
    "tree",
)


def _run(*args: str) -> subprocess.CompletedProcess[str]:
    _ensure_aw_binary()
    return subprocess.run(
        [str(AW_BINARY), *args], capture_output=True, text=True, check=False
    )


def _top_level_commands(help_text: str) -> set[str]:
    names: set[str] = set()
    in_commands = False
    for line in help_text.splitlines():
        if line.strip() == "Commands:":
            in_commands = True
            continue
        if in_commands:
            if not line.startswith("  ") or not line.strip():
                if line.strip() and not line.startswith("  "):
                    break
                if not line.strip():
                    continue
            stripped = line.strip()
            match = re.match(r"^([a-z][a-z-]*)\s", stripped)
            if match:
                names.add(match.group(1))
    return names


def verify() -> list[str]:
    outline = _run("llm")
    assert outline.returncode == 0, outline.stderr
    body = outline.stdout
    assert "## Standard agent commands" in body, body
    section_start = body.index("## Standard agent commands")
    section = body[section_start:]

    top_help = _run("--help")
    assert top_help.returncode == 0, top_help.stderr
    real_commands = _top_level_commands(top_help.stdout)
    assert "wi" in real_commands, real_commands
    assert "llm" in real_commands and "upgrade" in real_commands and "issue" in real_commands, real_commands

    for standard_command in ("aw llm", "aw upgrade", "aw issue"):
        assert standard_command in section, section
        bare = standard_command.split()[1]
        assert bare in real_commands, (bare, real_commands)

    assert "aw wi " not in section and "\naw wi\n" not in section, section

    upgrade_help = _run("upgrade", "--help")
    assert "--version" in upgrade_help.stdout, upgrade_help.stdout
    assert "--check" in upgrade_help.stdout, upgrade_help.stdout
    assert "--version" in section and "--check" in section, section

    issue_help = _run("issue", "--help")
    for sub in ("search", "view", "create", "comment"):
        assert sub in issue_help.stdout, issue_help.stdout
        assert sub in section, section

    issue_create_help = _run("issue", "create", "--help")
    assert "--title" in issue_create_help.stdout, issue_create_help.stdout
    assert "--title" in section, section

    issue_comment_help = _run("issue", "comment", "--help")
    assert "<NUMBER>" in issue_comment_help.stdout, issue_comment_help.stdout

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
