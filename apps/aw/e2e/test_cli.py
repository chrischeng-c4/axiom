"""CLI surface contract for the typer app.

`.claude/aw/verification/check_next_command.py` owns one half of the printed
protocol: every `next.command:` line an engine script prints must parse in the
engine's own argparse. This file owns the other half its docstring assigns
here: the typer surface accepts each group/verb/option, delegates to the
engine module named by its group, and rebuilds an argv that engine's argparse
accepts -- measured with `_delegate` stubbed, so no verb body ever runs and no
case touches the tracker or the tree.
"""

from __future__ import annotations

import contextlib
import importlib
import io
import subprocess
import sys

import pytest
from typer.testing import CliRunner

import aw
from aw import main as cli


def run_aw(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, "-m", "aw.main", *args],
        capture_output=True, text=True,
    )


def test_no_args_prints_help_and_exits_nonzero() -> None:
    result = run_aw()
    assert result.returncode != 0
    assert "Usage" in result.stdout or "Usage" in result.stderr


def test_version_prints_project_version() -> None:
    result = run_aw("version")
    assert result.returncode == 0
    assert result.stdout.strip() == aw.__version__


# --- delegation contract ---------------------------------------------------

runner = CliRunner()

SHA = "a" * 40
DIGEST = "b" * 64

# One row per typer command: (group, verb, cli tokens, expected delegation).
# The delegated module is always the group name -- that equality is what lets
# check_next_command map a printed `aw <group> ...` back to `<group>.py`.
CASES: tuple[tuple[str, str, list[str], list[str]], ...] = (
    ("release-plan", "validate",
     ["release-plan", "validate", "--plan", "plan.json"],
     ["validate", "--plan", "plan.json"]),
    ("release-plan", "apply",
     ["release-plan", "apply", "--plan", "plan.json", "--project", "apps/demo",
      "--approved-digest", DIGEST],
     ["apply", "--plan", "plan.json", "--project", "apps/demo",
      "--approved-digest", DIGEST]),
    ("release-plan", "resume",
     ["release-plan", "resume", "--receipt", ".aw/release-plans/a/demo.json"],
     ["resume", "--receipt", ".aw/release-plans/a/demo.json"]),
    ("change", "skeleton",
     ["change", "skeleton", "--type", "feat"],
     ["skeleton", "--type", "feat"]),
    ("change", "bodydir",
     ["change", "bodydir", "--type", "docs"],
     ["bodydir", "--type", "docs"]),
    ("change", "fetch",
     ["change", "--repo", "owner/repo", "fetch", "42"],
     ["--repo", "owner/repo", "fetch", "42"]),
    ("change", "adopt",
     ["change", "adopt", ".aw/bodies/feat/staged.md", "42", "--type", "feat"],
     ["adopt", ".aw/bodies/feat/staged.md", "42", "--type", "feat"]),
    ("change", "validate",
     ["change", "validate", "--body-file", "body.md", "--type", "feat", "--json"],
     ["validate", "--body-file", "body.md", "--type", "feat", "--json"]),
    ("change", "validate",
     ["change", "validate", "42"],
     ["validate", "42"]),
    ("change", "show",
     ["change", "show", "42", "--json"],
     ["show", "42", "--json"]),
    ("change", "create",
     ["change", "create", "--title", "t", "--body-file", "body.md",
      "--type", "feat", "--milestone", "milestone:7", "--priority", "p1",
      "--project", "demo", "--dry-run"],
     ["create", "--title", "t", "--body-file", "body.md", "--type", "feat",
      "--priority", "p1", "--milestone", "milestone:7", "--project", "demo",
      "--dry-run"]),
    ("change", "lifecycle",
     ["change", "lifecycle", "42", "--leg", "e2e",
      "--commit", SHA, "--digest", DIGEST, "--dry-run"],
     ["lifecycle", "42", "--leg", "e2e",
      "--commit", SHA, "--digest", DIGEST, "--dry-run"]),
    ("change", "update",
     ["change", "update", "42", "--body-file", "body.md", "--title", "t",
      "--add-label", "app:demo", "--add-label", "priority:p1",
      "--remove-label", "priority:p2", "--milestone", "demo@0.2.0",
      "--dry-run"],
     ["update", "42", "--body-file", "body.md", "--title", "t",
      "--add-label", "app:demo", "--add-label", "priority:p1",
      "--remove-label", "priority:p2", "--milestone", "demo@0.2.0",
      "--dry-run"]),
    ("change", "update",
     ["change", "update", "42", "--remove-milestone"],
     ["update", "42", "--remove-milestone"]),
    ("change", "retype",
     ["change", "retype", "42", "--to", "fix", "--dry-run"],
     ["retype", "42", "--to", "fix", "--dry-run"]),
    ("change", "close",
     ["change", "close", "42", "--dry-run"],
     ["close", "42", "--dry-run"]),

    ("milestone", "skeleton",
     ["milestone", "skeleton"],
     ["skeleton"]),
    ("milestone", "validate",
     ["milestone", "--repo", "owner/repo", "validate", "demo@0.2.0", "--json"],
     ["--repo", "owner/repo", "validate", "demo@0.2.0", "--json"]),
    ("milestone", "validate",
     ["milestone", "validate", "--description-file", "desc.md",
      "--title", "demo@0.2.0", "--draft"],
     ["validate", "--description-file", "desc.md",
      "--title", "demo@0.2.0", "--draft"]),
    ("milestone", "show",
     ["milestone", "show", "milestone:7", "--json"],
     ["show", "milestone:7", "--json"]),
    ("milestone", "children",
     ["milestone", "children", "milestone:7"],
     ["children", "milestone:7"]),
    ("milestone", "reconcile",
     ["milestone", "reconcile", "milestone:7", "--json"],
     ["reconcile", "milestone:7", "--json"]),
    ("milestone", "order",
     ["milestone", "order", "milestone:7", "--open-only", "--json"],
     ["order", "milestone:7", "--open-only", "--json"]),
    ("milestone", "next",
     ["milestone", "next", "milestone:7", "--json"],
     ["next", "milestone:7", "--json"]),
    ("milestone", "versions",
     ["milestone", "versions", "--project", "demo", "--state", "closed", "--json"],
     ["versions", "--state", "closed", "--project", "demo", "--json"]),
    ("milestone", "next-version",
     ["milestone", "next-version", "demo", "--bump", "major", "--json"],
     ["next-version", "demo", "--bump", "major", "--json"]),
    ("milestone", "create",
     ["milestone", "create", "--title", "demo@0.2.0",
      "--description-file", "desc.md", "--due-on", "2026-09-30",
      "--draft", "--dry-run"],
     ["create", "--title", "demo@0.2.0", "--description-file", "desc.md",
      "--due-on", "2026-09-30", "--draft", "--dry-run"]),
    ("milestone", "update",
     ["milestone", "update", "milestone:7", "--title", "demo@0.2.0",
      "--description-file", "desc.md", "--clear-due-on", "--draft", "--dry-run"],
     ["update", "milestone:7", "--title", "demo@0.2.0",
      "--description-file", "desc.md", "--clear-due-on", "--draft", "--dry-run"]),
    ("milestone", "close",
     ["milestone", "close", "milestone:7", "--dry-run"],
     ["close", "milestone:7", "--dry-run"]),

    ("e2e", "start",
     ["e2e", "--project", "demo", "start", "42"],
     ["--project", "demo", "start", "42"]),
    ("e2e", "verify",
     ["e2e", "--project", "demo", "verify", "42"],
     ["--project", "demo", "verify", "42"]),
    ("e2e", "test",
     ["e2e", "--project", "demo", "test", "42"],
     ["--project", "demo", "test", "42"]),
    ("e2e", "commit",
     ["e2e", "--project", "demo", "commit", "42", "--dry-run"],
     ["--project", "demo", "commit", "42", "--dry-run"]),

    ("impl", "start",
     ["impl", "--project", "demo", "start", "42"],
     ["--project", "demo", "start", "42"]),
    ("impl", "red",
     ["impl", "--project", "demo", "red", "42"],
     ["--project", "demo", "red", "42"]),
    ("impl", "verify",
     ["impl", "--project", "demo", "verify", "42"],
     ["--project", "demo", "verify", "42"]),
    ("impl", "test",
     ["impl", "--project", "demo", "test", "42"],
     ["--project", "demo", "test", "42"]),
    ("impl", "commit",
     ["impl", "--project", "demo", "commit", "42", "--dry-run"],
     ["--project", "demo", "commit", "42", "--dry-run"]),

    ("maint", "start",
     ["maint", "--project", "demo", "start", "42"],
     ["--project", "demo", "start", "42"]),
    ("maint", "record",
     ["maint", "--project", "demo", "record", "42", "--when", "after",
      "--command", "cargo test -p demo", "--exit", "0",
      "--output-file", "gate.out"],
     ["--project", "demo", "record", "42", "--when", "after",
      "--command", "cargo test -p demo", "--exit", "0",
      "--output-file", "gate.out"]),
    ("maint", "verify",
     ["maint", "--project", "demo", "verify", "42"],
     ["--project", "demo", "verify", "42"]),
    ("maint", "commit",
     ["maint", "--project", "demo", "commit", "42", "--dry-run"],
     ["--project", "demo", "commit", "42", "--dry-run"]),

    ("wis", "gap",
     ["wis", "gap", "demo", "--repo", "owner/repo", "--format", "json"],
     ["gap", "demo", "--format", "json", "--repo", "owner/repo"]),
    ("wis", "gap",
     ["wis", "gap", "demo"],
     ["gap", "demo", "--format", "text"]),

    ("meta", "check",
     ["meta", "check", "--repo", "/some/checkout", "--rule", "M2",
      "--rule", "M7", "--path", "CLAUDE.md", "--format", "json"],
     ["check", "--format", "json", "--repo", "/some/checkout",
      "--rule", "M2", "--rule", "M7", "--path", "CLAUDE.md"]),
    ("meta", "check",
     ["meta", "check"],
     ["check", "--format", "text"]),

    ("metadoc", "check",
     ["metadoc", "check", "demo", "--format", "json"],
     ["check", "demo", "--format", "json"]),
    ("metadoc", "commit",
     ["metadoc", "commit", "demo", "--why", "why.md", "--dry-run"],
     ["commit", "demo", "--why", "why.md", "--dry-run"]),
)


def delegated(monkeypatch: pytest.MonkeyPatch,
              tokens: list[str]) -> tuple[str, list[str]]:
    """Invoke the app with `_delegate` stubbed; return what it delegated."""
    calls: list[tuple[str, list[str]]] = []

    def capture(module: str, argv: list[str]) -> None:
        calls.append((module, list(argv)))

    monkeypatch.setattr(cli, "_delegate", capture)
    result = runner.invoke(cli.app, tokens)
    assert result.exit_code == 0, result.output
    assert len(calls) == 1, calls
    return calls[0]


def engine_refusal(module_name: str, argv: list[str]) -> str | None:
    """`None` if the engine module's argparse accepts argv, else the reason.

    Same probe as check_next_command.accepts: a module exposing
    `build_parser` is parsed directly; one whose parser is local to `main`
    has every `cmd_*` global stubbed first, so `set_defaults(func=cmd_x)`
    resolves the stub when `main` builds the parser and no verb body runs.
    """
    scripts = str(cli._SCRIPTS)
    if scripts not in sys.path:
        sys.path.insert(0, scripts)
    module = importlib.import_module(module_name.replace("-", "_"))
    err = io.StringIO()
    originals = {name: getattr(module, name)
                 for name in vars(module) if name.startswith("cmd_")}
    try:
        with contextlib.redirect_stderr(err), \
                contextlib.redirect_stdout(io.StringIO()):
            if hasattr(module, "build_parser"):
                module.build_parser().parse_args(argv)
            else:
                for name in originals:
                    setattr(module, name, lambda *a, **k: 0)
                module.main(argv)
    except SystemExit as exit_:
        if exit_.code:
            reason = next((line for line in err.getvalue().splitlines()
                           if "error:" in line), f"exit {exit_.code}")
            return reason.strip()
    except Exception as exc:  # a broken parser is also a refusal
        return f"{type(exc).__name__}: {exc}"
    finally:
        for name, fn in originals.items():
            setattr(module, name, fn)
    return None


@pytest.mark.parametrize(
    ("group", "verb", "tokens", "expected"),
    CASES,
    ids=[f"{group}-{verb}-{i}" for i, (group, verb, _, _) in enumerate(CASES)],
)
def test_delegates_faithfully(monkeypatch: pytest.MonkeyPatch, group: str,
                              verb: str, tokens: list[str],
                              expected: list[str]) -> None:
    module, argv = delegated(monkeypatch, tokens)
    assert module == group
    assert verb in argv
    assert argv == expected
    refusal = engine_refusal(module, argv)
    assert refusal is None, refusal


def test_cases_cover_every_registered_command() -> None:
    registered = {
        group.name: {command.name
                     for command in group.typer_instance.registered_commands}
        for group in cli.app.registered_groups
    }
    covered: dict[str, set[str]] = {}
    for group, verb, _tokens, _expected in CASES:
        covered.setdefault(group, set()).add(verb)
    assert covered == registered
    assert {command.name or command.callback.__name__
            for command in cli.app.registered_commands} == {"version"}


def test_engine_probe_can_refuse() -> None:
    # Instrument control: validation lives in the engine's argparse, and the
    # probe reports it -- a typo the plain-str typer options let through.
    refusal = engine_refusal("change", [
        "lifecycle", "42", "--leg", "nope", "--commit", SHA, "--digest", DIGEST,
    ])
    assert refusal is not None and "--leg" in refusal
