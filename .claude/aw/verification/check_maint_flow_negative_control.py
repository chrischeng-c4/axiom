#!/usr/bin/env python3
"""Plant one forbidden maintenance shape per temporary checkout.

Each control asserts the specific refusal text.  A script that refused every
input would therefore fail this gate's positive companion, while a control
that tripped the wrong precondition would fail here.
"""
from __future__ import annotations

import json
import pathlib
import tempfile

import check_maint_flow as support


Path = pathlib.Path
fails: list[str] = []


def check(name: str, ok: bool, detail: str = "") -> None:
    print(f"{'PASS' if ok else 'FAIL'} {name}")
    if not ok:
        fails.append(name)
        if detail:
            for line in detail.splitlines():
                print(f"     {line}")


def refuses(name: str, proc, needle: str) -> None:
    output = proc.stdout + proc.stderr
    check(name, proc.returncode != 0 and needle in output, output)


def behavior_type(tmp: Path) -> None:
    repo, _output = support.build(
        tmp, "feat", ["apps/demo/src/lib.rs"],
    )
    refuses(
        "behavior delivery cannot start maint",
        support.run(repo, "start", str(support.WI)),
        "maint accepts only",
    )


def double_type(tmp: Path) -> None:
    repo, _output = support.build(tmp, "docs", ["apps/demo/README.md"])
    receipt = repo / f".aw/workitems/deliveries/{support.WI}.json"
    value = json.loads(receipt.read_text())
    value["labels"].append("type:test")
    receipt.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
    refuses(
        "double type cannot start maint",
        support.run(repo, "start", str(support.WI)),
        "needs exactly type:docs",
    )


def wrong_phase(tmp: Path) -> None:
    repo, _output = support.build(tmp, "docs", ["apps/demo/README.md"])
    receipt = repo / f".aw/workitems/deliveries/{support.WI}.json"
    value = json.loads(receipt.read_text())
    value["labels"] = [
        "phase:impl",
        *[label for label in value["labels"] if label != "phase:created"],
    ]
    receipt.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
    refuses(
        "maintenance starts only from phase:created",
        support.run(repo, "start", str(support.WI)),
        "must be in phase:created",
    )


def dirty_start(tmp: Path) -> None:
    repo, _output = support.build(tmp, "docs", ["apps/demo/README.md"])
    target = repo / "apps/demo/README.md"
    target.write_text(target.read_text() + "dirty\n", encoding="utf-8")
    refuses(
        "dirty tree cannot create a baseline",
        support.run(repo, "start", str(support.WI)),
        "start needs a clean working tree",
    )
    check(
        "dirty start wrote no baseline record",
        not (repo / f".aw/maint/{support.WI}.json").exists(),
    )


def stale_staged_body(tmp: Path) -> None:
    repo, _output = support.build(tmp, "docs", ["apps/demo/README.md"])
    staged = repo / f".aw/workitems/deliveries/{support.WI}.md"
    staged.write_text(staged.read_text() + "\nchanged after fetch\n", encoding="utf-8")
    refuses(
        "body bytes must match the fetch receipt",
        support.run(repo, "start", str(support.WI)),
        "do not match the tracker receipt",
    )


def refactor_without_before(tmp: Path) -> None:
    repo, output = support.build(tmp, "refactor", ["apps/demo/src/lib.rs"])
    support.run(repo, "start", str(support.WI))
    target = repo / "apps/demo/src/lib.rs"
    target.write_text(target.read_text().replace("    1\n", "    1_u32\n"), encoding="utf-8")
    refuses(
        "refactor cannot record after before",
        support.record_after(repo, output),
        "record the before result",
    )


def undeclared_command(tmp: Path) -> None:
    path = "apps/demo/src/tests.rs"
    repo, output = support.build(tmp, "test", [path])
    support.run(repo, "start", str(support.WI))
    target = repo / path
    target.write_text(target.read_text() + "\n// test note\n", encoding="utf-8")
    proc = support.run(
        repo, "record", str(support.WI), "--when", "after",
        "--command", "python3 another_gate.py", "--exit", "0",
        "--output-file", str(output),
    )
    refuses("record binds the exact declared command", proc, "not an exact Acceptance command")


def stale_after_record(tmp: Path) -> None:
    path = "apps/demo/src/tests.rs"
    repo, output = support.build(tmp, "test", [path])
    support.run(repo, "start", str(support.WI))
    target = repo / path
    target.write_text(target.read_text() + "\n// first test change\n", encoding="utf-8")
    support.record_after(repo, output)
    target.write_text(target.read_text() + "// drift after evidence\n", encoding="utf-8")
    refuses(
        "after evidence cannot survive diff drift",
        support.run(repo, "verify", str(support.WI)),
        "describes a different tree",
    )


def test_product_code(tmp: Path) -> None:
    path = "apps/demo/src/lib.rs"
    repo, output = support.build(tmp, "test", [path])
    support.run(repo, "start", str(support.WI))
    target = repo / path
    target.write_text(target.read_text().replace("    1\n", "    2\n"), encoding="utf-8")
    support.record_after(repo, output)
    refuses(
        "type:test refuses product implementation",
        support.run(repo, "verify", str(support.WI)),
        "changed product code",
    )


def docs_product_code(tmp: Path) -> None:
    path = "apps/demo/src/lib.rs"
    repo, output = support.build(tmp, "docs", [path])
    support.run(repo, "start", str(support.WI))
    target = repo / path
    target.write_text(target.read_text().replace("    1\n", "    2\n"), encoding="utf-8")
    support.record_after(repo, output)
    refuses(
        "type:docs refuses executable hunks",
        support.run(repo, "verify", str(support.WI)),
        "changed non-document code",
    )


def quoted_cfg_is_not_test_scope(tmp: Path) -> None:
    path = "apps/demo/src/lib.rs"
    repo, output = support.build(tmp, "test", [path])
    target = repo / path
    target.write_text(
        target.read_text()
        + '\npub const FIXTURE: &str = r#"\n#[cfg(test)]\nmod quoted {\n    product = 1\n}\n"#;\n',
        encoding="utf-8",
    )
    support.git(repo, "add", path)
    support.git(repo, "commit", "-q", "-m", "quoted cfg fixture")
    support.run(repo, "start", str(support.WI))
    target.write_text(target.read_text().replace("    product = 1", "    product = 2"), encoding="utf-8")
    support.record_after(repo, output)
    refuses(
        "quoted cfg marker does not widen test scope",
        support.run(repo, "verify", str(support.WI)),
        "changed product code",
    )


def quoted_comment_is_not_docs_scope(tmp: Path) -> None:
    path = "apps/demo/src/lib.rs"
    repo, output = support.build(tmp, "docs", [path])
    target = repo / path
    target.write_text(
        target.read_text()
        + '\npub const FIXTURE: &str = r#"\n// quoted documentation\n"#;\n',
        encoding="utf-8",
    )
    support.git(repo, "add", path)
    support.git(repo, "commit", "-q", "-m", "quoted docs fixture")
    support.run(repo, "start", str(support.WI))
    target.write_text(
        target.read_text().replace("// quoted documentation", "// changed quoted text"),
        encoding="utf-8",
    )
    support.record_after(repo, output)
    refuses(
        "quoted comment marker does not widen docs scope",
        support.run(repo, "verify", str(support.WI)),
        "changed non-document code",
    )


def chore_product_src(tmp: Path) -> None:
    path = "apps/demo/src/lib.rs"
    repo, output = support.build(tmp, "chore", [path])
    support.run(repo, "start", str(support.WI))
    target = repo / path
    target.write_text(target.read_text().replace("    1\n", "    2\n"), encoding="utf-8")
    support.record_after(repo, output)
    refuses(
        "type:chore refuses product src even when declared",
        support.run(repo, "verify", str(support.WI)),
        "must not change product src",
    )


def nonzero_gate(tmp: Path) -> None:
    path = "apps/demo/README.md"
    repo, output = support.build(tmp, "docs", [path])
    support.run(repo, "start", str(support.WI))
    target = repo / path
    target.write_text(target.read_text() + "\nNew product note.\n", encoding="utf-8")
    support.record_after(repo, output, exit_code=7)
    refuses(
        "nonzero controller gate cannot verify",
        support.run(repo, "verify", str(support.WI)),
        "exited 7, not 0",
    )


def no_automatic_test_verb(tmp: Path) -> None:
    path = "apps/demo/src/tests.rs"
    repo, _output = support.build(tmp, "test", [path])
    proc = support.run(repo, "test", str(support.WI))
    check(
        "there is no automatic test verb",
        proc.returncode != 0 and "invalid choice" in proc.stderr,
        proc.stdout + proc.stderr,
    )
    check(
        "refused verb did not execute the issue command",
        not (repo / "COMMAND_RAN").exists(),
    )


def main() -> int:
    cases = (
        behavior_type,
        double_type,
        wrong_phase,
        dirty_start,
        stale_staged_body,
        refactor_without_before,
        undeclared_command,
        stale_after_record,
        test_product_code,
        docs_product_code,
        quoted_cfg_is_not_test_scope,
        quoted_comment_is_not_docs_scope,
        chore_product_src,
        nonzero_gate,
        no_automatic_test_verb,
    )
    for case in cases:
        with tempfile.TemporaryDirectory(prefix=f"maint-negative-{case.__name__}-") as raw:
            case(Path(raw))
    print(f"\n{len(cases)} negative controls; {len(fails)} failure(s)")
    return 1 if fails else 0


if __name__ == "__main__":
    raise SystemExit(main())
