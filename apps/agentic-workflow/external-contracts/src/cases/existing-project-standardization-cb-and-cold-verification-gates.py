"""Black-box contract for `aw cb gen --force-regen --verify-cold` (#3310)."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import run_aw

CASE_ID = "existing-project-standardization-cb-and-cold-verification-gates"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "cb-and-cold-verification-gates"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-cb-and-cold-verification-gates"
)
ASSERTIONS = (
    "aw cb gen --force-regen --verify-cold --project <p>, run against a clean "
    "force-regenerated tree, rebuilds every expected target from an isolated "
    "TD-only scratch copy and reports the one impl_mode: codegen Changes entry "
    "as its cold_rebuild denominator and numerator (files 1/1) while never "
    "counting the sibling impl_mode: hand-written entry, proving cold-rebuild "
    "targets are derived from codegen Changes entries specifically",
    "--verify-cold used without --force-regen is rejected outright with the "
    "literal '--verify-cold is only supported with --force-regen' error before "
    "any project resolution runs, and --force-regen --verify --verify-cold "
    "together is rejected with the literal '--verify-cold cannot be combined "
    "with --verify' error, proving cold verification is a distinct, mutually "
    "exclusive force-regen mode rather than a modifier compatible with the "
    "live --verify pipeline",
)

_MANUAL_RS = 'pub fn authored() { println!("keep spacing"); }\n'

_SPEC = """---
id: tool-apps-tool
fill_sections: [schema, changes]
---

# Tool

## Schema
<!-- type: schema lang: yaml -->

```yaml
source_units:
  - path: apps/tool/llms.txt
    generator_primitives: [project_root_llms]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tool/llms.txt
    action: modify
    section: schema
    impl_mode: codegen
  - path: apps/tool/src/manual.rs
    action: modify
    section: schema
    impl_mode: hand-written
```
"""

_PLACEHOLDER_LLMS = (
    "<!-- SPEC-MANAGED: apps/tool/tech-design/semantic/tool-apps-tool.md#schema -->\n"
    "<!-- CODEGEN-BEGIN -->\n"
    "TODO: generic project context placeholder\n"
    "<!-- CODEGEN-END -->\n"
)


def _git(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(
        ["git", *args], cwd=root, capture_output=True, text=True, check=False
    )
    if completed.returncode != 0:
        raise AssertionError(f"git {' '.join(args)} failed: {completed.stderr}")
    return completed


def _write_fixture(root: Path) -> None:
    (root / "aw.toml").write_text(
        '[[projects]]\n'
        'name = "tool"\n'
        'path = "apps/tool"\n'
        'td_path = "apps/tool/tech-design"\n'
        'cap_path = "apps/tool/README.md"\n'
        'label = "app:tool"\n\n'
        "[[projects.workspaces]]\n"
        'name = "tool"\n'
        'paths = ["apps/tool/**"]\n'
        'target = "rust"\n'
        'test_cmd = "cargo test -p tool"\n',
        encoding="utf-8",
    )
    (root / "apps/tool/src").mkdir(parents=True)
    (root / "apps/tool/tech-design/semantic").mkdir(parents=True)
    (root / "apps/tool/tech-design/src/tool").mkdir(parents=True)
    (root / "apps/tool/Cargo.toml").write_text(
        '[package]\nname = "tool"\nversion = "0.1.0"\nedition = "2021"\n',
        encoding="utf-8",
    )
    (root / "apps/tool/README.md").write_text("# Tool\n", encoding="utf-8")
    (root / "apps/tool/src/manual.rs").write_text(_MANUAL_RS, encoding="utf-8")
    # `aw td lock` runs its Python TD compiler check unconditionally, which
    # requires at least one real `.py` TD module below `src/` regardless of
    # the workspace `target` setting.
    (root / "apps/tool/tech-design/src/tool/policy.py").write_text(
        '__aw_artifact_id__ = "artifact:policy/evaluate"\n\nclass Policy:\n    pass\n',
        encoding="utf-8",
    )
    (root / "apps/tool/tech-design/pyproject.toml").write_text(
        '[project]\nname = "tool-tech-design"\nversion = "0.1.0"\n'
        'requires-python = ">=3.11"\n',
        encoding="utf-8",
    )
    # A dependency-free synthetic lock so `aw td lock` never shells to `uv`
    # or the network, mirroring `write_python_artifact_lock`'s own shape.
    (root / "apps/tool/tech-design/uv.lock").write_text(
        "version = 1\nrevision = 3\nrequires-python = \">=3.11\"\n\n"
        '[[package]]\nname = "tool-tech-design"\nversion = "0.1.0"\n'
        'source = { virtual = "." }\n',
        encoding="utf-8",
    )
    (root / "apps/tool/tech-design/semantic/tool-apps-tool.md").write_text(
        _SPEC, encoding="utf-8"
    )
    (root / "apps/tool/llms.txt").write_text(_PLACEHOLDER_LLMS, encoding="utf-8")

    _git(root, "init")
    _git(root, "config", "user.email", "fixture@example.com")
    _git(root, "config", "user.name", "Fixture")
    _git(root, "add", "-A")
    _git(root, "commit", "-m", "fixture")


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-ec-cold-verify-") as raw_root:
        root = Path(raw_root)
        _write_fixture(root)

        lock = run_aw(root, "td", "lock", "--project", "tool")
        assert "td.lock" in lock.stdout, lock.stdout
        _git(root, "add", "-A")
        _git(root, "commit", "-m", "lock")

        regen = run_aw(root, "cb", "gen", "--force-regen", "--project", "tool")
        assert regen.returncode == 0, regen.stderr

        cold = run_aw(root, "cb", "gen", "--force-regen", "--verify-cold", "--project", "tool")
        combined_cold_output = cold.stdout + cold.stderr
        assert "cold_rebuild: files 1/1" in combined_cold_output, combined_cold_output
        assert "1 spec(s), 1 source root(s)" in combined_cold_output, combined_cold_output

        bare = run_aw(root, "cb", "gen", "--verify-cold", expect_success=False)
        assert (
            "--verify-cold is only supported with --force-regen" in bare.stderr
        ), bare.stderr

        combo = run_aw(
            root,
            "cb",
            "gen",
            "--force-regen",
            "--project",
            "tool",
            "--verify",
            "--verify-cold",
            expect_success=False,
        )
        assert (
            "--verify-cold cannot be combined with --verify" in combo.stderr
        ), combo.stderr

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
