#!/usr/bin/env python3
"""Exercise every admitted maintenance shape in an isolated checkout.

The fixture owns its git repository, staged delivery receipt, and output files.
No case reads or changes the caller's working tree.  Commands in the GHAN body
are inert strings: the fixture writes the output capture itself, then asks
``maint.py record`` to hash it.
"""
from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from _paths import pinned_interpreter  # noqa: E402


Path = pathlib.Path
HERE = Path(__file__).resolve().parent
MAINT = HERE.parent / "scripts" / "maint.py"
PYTHON = pinned_interpreter()
GIT = ("git", "-c", "core.fsmonitor=false")
WI = 41
GATE = "python3 tools/check_demo.py"
PIPE_GATE = "python3 tools/check_demo.py | tee /tmp/demo-gate.log"

fails: list[str] = []


def check(name: str, ok: bool, detail: str = "") -> None:
    print(f"{'PASS' if ok else 'FAIL'} {name}")
    if not ok:
        fails.append(name)
        if detail:
            for line in detail.splitlines():
                print(f"     {line}")


def git(repo: Path, *args: str, check_result: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [*GIT, *args], cwd=repo, capture_output=True, text=True,
        check=check_result,
    )


def run(repo: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [*PYTHON, str(MAINT), "--project", "demo", *args],
        cwd=repo, capture_output=True, text=True,
    )


def body(paths: list[str], gate: str = GATE) -> str:
    points = "\n".join(f"- `{path}` is the bounded write target." for path in paths)
    table_gate = gate.replace("|", "\\|")
    return f"""## Goal

The demo maintenance change keeps the declared project contract verifiable.

## How

### Verified premises

- `apps/demo/src/lib.rs:1` contains the current fixture behavior.

### Change points

{points}

### Frozen decisions

No other path or behavior is in scope.

## Acceptance

| # | command | current | target | why it cannot hold by accident |
|---|---|---|---|---|
| 1 | `{table_gate}` | The maintenance change is absent. | The maintenance gate exits zero. | The command reads the changed fixture. |

### Negative control

- Corrupt the changed fixture. The gate must fail. Restore sha256 `{'0' * 64}`.

## Never

This addresses the worker implementing this work item.

### Must not touch

- `apps/other/src/lib.rs`

### Must not do

- Do not change tracker state.
"""


def stage(repo: Path, kind: str, paths: list[str], gate: str = GATE) -> None:
    text = body(paths, gate)
    directory = repo / ".aw/workitems/deliveries"
    directory.mkdir(parents=True, exist_ok=True)
    (directory / f"{WI}.md").write_text(text, encoding="utf-8")
    receipt = {
        "iid": WI,
        "type": kind,
        "flow": "maintenance" if kind in {"refactor", "test", "docs", "chore"} else "behavior",
        "state": "OPEN",
        "milestone": 9,
        "labels": [f"type:{kind}", "phase:created", "app:demo"],
        "updated_at": "2026-08-31T00:00:00Z",
        "body_sha256": hashlib.sha256(text.encode("utf-8")).hexdigest(),
    }
    (directory / f"{WI}.json").write_text(
        json.dumps(receipt, indent=2) + "\n", encoding="utf-8",
    )


def build(tmp: Path, kind: str, paths: list[str], gate: str = GATE) -> tuple[Path, Path]:
    repo = tmp / "checkout"
    (repo / "apps/demo/src").mkdir(parents=True)
    (repo / "apps/demo/docs").mkdir(parents=True)
    (repo / "tools").mkdir(parents=True)
    (repo / "aw.toml").write_text("[aw]\n", encoding="utf-8")
    (repo / ".gitignore").write_text(".aw/\n", encoding="utf-8")
    (repo / "apps/demo/Cargo.toml").write_text(
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        encoding="utf-8",
    )
    (repo / "apps/demo/README.md").write_text(
        "# demo\n\nThe current product contract.\n", encoding="utf-8",
    )
    (repo / "apps/demo/docs/guide.md").write_text(
        "# Guide\n\nCurrent guidance.\n", encoding="utf-8",
    )
    (repo / "apps/demo/src/lib.rs").write_text(
        "/// Return the current value.\n"
        "pub fn value() -> u32 {\n"
        "    1\n"
        "}\n\n"
        "#[cfg(test)]\n"
        "mod inline_tests {\n"
        "    #[test]\n"
        "    fn value_is_one() {\n"
        "        assert_eq!(super::value(), 1);\n"
        "    }\n"
        "}\n",
        encoding="utf-8",
    )
    (repo / "apps/demo/src/tests.rs").write_text(
        "#[test]\nfn existing_test() { assert!(true); }\n", encoding="utf-8",
    )
    # If maint.py ever executes this body command, the marker exposes it.
    (repo / "tools/check_demo.py").write_text(
        "from pathlib import Path\nPath('COMMAND_RAN').write_text('ran')\n",
        encoding="utf-8",
    )
    git(repo.parent, "init", "-q", "-b", "main", str(repo))
    git(repo, "config", "user.name", "maint-gate")
    git(repo, "config", "user.email", "maint-gate@example.invalid")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "fixture")
    stage(repo, kind, paths, gate)
    output = tmp / "gate-output.txt"
    output.write_text("controller captured this output\n", encoding="utf-8")
    return repo, output


def expect_ok(name: str, proc: subprocess.CompletedProcess[str]) -> None:
    check(name, proc.returncode == 0, proc.stdout + proc.stderr)


def record_after(repo: Path, output: Path, gate: str = GATE, exit_code: int = 0) -> subprocess.CompletedProcess[str]:
    return run(
        repo, "record", str(WI), "--when", "after", "--command", gate,
        "--exit", str(exit_code), "--output-file", str(output),
    )


def positive_refactor(tmp: Path) -> None:
    repo, output = build(tmp, "refactor", ["apps/demo/src/lib.rs"])
    expect_ok("refactor start", run(repo, "start", str(WI)))
    expect_ok(
        "refactor before record",
        run(
            repo, "record", str(WI), "--when", "before", "--command", GATE,
            "--exit", "0", "--output-file", str(output),
        ),
    )
    source = repo / "apps/demo/src/lib.rs"
    source.write_text(source.read_text().replace("    1\n", "    1_u32\n"), encoding="utf-8")
    expect_ok("refactor after record", record_after(repo, output))
    expect_ok("refactor verify", run(repo, "verify", str(WI)))
    dry = run(repo, "commit", str(WI), "--dry-run")
    required = (
        "Maint-Type: refactor", "Maint-Base:", "Maint-Gates:",
        "Maint-Contract:", "Maint-Change-Digest:",
    )
    check(
        "commit dry-run carries the complete maintenance contract",
        dry.returncode == 0
        and all(token in dry.stdout for token in required)
        and "<commit-sha>" not in dry.stdout,
        dry.stdout + dry.stderr,
    )
    landed = run(repo, "commit", str(WI))
    message = git(repo, "log", "-1", "--format=%B").stdout
    check(
        "landed commit preserves evidence and prints complete follow-ups",
        landed.returncode == 0
        and all(token in message for token in required)
        and "next.command: change.py lifecycle" in landed.stdout
        and "after.lifecycle.command: change.py close" in landed.stdout,
        landed.stdout + landed.stderr + message,
    )
    check(
        "maintenance never executed the body command",
        not (repo / "COMMAND_RAN").exists(),
    )


def positive_test_file(tmp: Path) -> None:
    path = "apps/demo/src/tests.rs"
    repo, output = build(tmp, "test", [path])
    expect_ok("test-file start", run(repo, "start", str(WI)))
    target = repo / path
    target.write_text(
        target.read_text() + "\n#[test]\nfn added_test() { assert_eq!(2 + 2, 4); }\n",
        encoding="utf-8",
    )
    expect_ok("test-file after record", record_after(repo, output))
    expect_ok("test-file verify", run(repo, "verify", str(WI)))


def positive_test_section(tmp: Path) -> None:
    path = "apps/demo/src/lib.rs"
    repo, output = build(tmp, "test", [path])
    expect_ok("test-section start", run(repo, "start", str(WI)))
    target = repo / path
    target.write_text(
        target.read_text().replace(
            "        assert_eq!(super::value(), 1);",
            "        assert_eq!(super::value(), 1_u32);",
        ),
        encoding="utf-8",
    )
    expect_ok("test-section after record", record_after(repo, output))
    expect_ok("test-section verify", run(repo, "verify", str(WI)))


def positive_docs_product(tmp: Path) -> None:
    path = "apps/demo/README.md"
    repo, output = build(tmp, "docs", [path])
    expect_ok("product-doc start", run(repo, "start", str(WI)))
    target = repo / path
    target.write_text(target.read_text() + "\nCurrent operator note.\n", encoding="utf-8")
    expect_ok("product-doc after record", record_after(repo, output))
    expect_ok("product-doc verify", run(repo, "verify", str(WI)))


def positive_docs_comment(tmp: Path) -> None:
    path = "apps/demo/src/lib.rs"
    repo, output = build(tmp, "docs", [path])
    expect_ok("source-doc start", run(repo, "start", str(WI)))
    target = repo / path
    target.write_text(
        target.read_text().replace(
            "/// Return the current value.", "/// Return the stable current value.",
        ),
        encoding="utf-8",
    )
    expect_ok("source-doc after record", record_after(repo, output))
    expect_ok("source-doc verify", run(repo, "verify", str(WI)))


def positive_chore(tmp: Path) -> None:
    path = "apps/demo/Cargo.toml"
    repo, output = build(tmp, "chore", [path], PIPE_GATE)
    expect_ok("chore start", run(repo, "start", str(WI)))
    target = repo / path
    target.write_text(target.read_text() + "\n[dev-dependencies]\n", encoding="utf-8")
    expect_ok("chore after record", record_after(repo, output, PIPE_GATE))
    expect_ok("chore verify", run(repo, "verify", str(WI)))


def main() -> int:
    if not MAINT.is_file():
        check("maintenance script exists", False, str(MAINT))
        return 1
    cases = (
        positive_refactor,
        positive_test_file,
        positive_test_section,
        positive_docs_product,
        positive_docs_comment,
        positive_chore,
    )
    for case in cases:
        with tempfile.TemporaryDirectory(prefix=f"maint-positive-{case.__name__}-") as raw:
            case(Path(raw))
    print(f"\n{len(cases)} positive maintenance shapes; {len(fails)} failure(s)")
    return 1 if fails else 0


if __name__ == "__main__":
    raise SystemExit(main())
