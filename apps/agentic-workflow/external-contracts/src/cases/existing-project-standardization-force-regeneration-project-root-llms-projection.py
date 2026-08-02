"""Black-box contract for the force-regen TD-first project-root llms emitter (#3310)."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import run_aw

CASE_ID = "existing-project-standardization-force-regeneration-project-root-llms-projection"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "force-regeneration-project-root-llms-projection"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-force-regeneration-project-root-llms-projection"
)
ASSERTIONS = (
    "aw cb gen --force-regen --project <p> replaces a generic project-root llms "
    "placeholder with exactly one <!-- CODEGEN-BEGIN/END --> block whose canonical "
    "TD-first content leads with a '# <project> Agent Context' heading and orders "
    "'## Tech Design' before '## Capability Map', proving the public "
    "force-regeneration path shares the same TD-first project-root llms emitter "
    "used by replay/cold verification instead of a generic placeholder writer",
    "the same force-regen pass leaves a sibling Changes entry marked "
    "impl_mode: hand-written byte-for-byte untouched and auto-commits its own "
    "working tree with a 'Lifecycle-Stage: Cb-Force-Regen' trailer, leaving git "
    "status clean, proving HANDWRITE siblings are preserved rather than "
    "reformatted alongside the regenerated CODEGEN target",
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
    with tempfile.TemporaryDirectory(prefix="aw-ec-force-regen-llms-") as raw_root:
        root = Path(raw_root)
        _write_fixture(root)

        # `aw cb gen --force-regen` refuses to run against a dirty TD lock, so
        # the fixture must lock (and commit the lock) before regenerating.
        lock = run_aw(root, "td", "lock", "--project", "tool")
        assert "td.lock" in lock.stdout, lock.stdout
        _git(root, "add", "-A")
        _git(root, "commit", "-m", "lock")

        regen = run_aw(root, "cb", "gen", "--force-regen", "--project", "tool")
        assert "1 file update(s)" in regen.stdout, regen.stdout

        generated = (root / "apps/tool/llms.txt").read_text(encoding="utf-8")
        assert generated.count("<!-- CODEGEN-BEGIN -->") == 1, generated
        assert generated.count("<!-- CODEGEN-END -->") == 1, generated
        assert "# tool Agent Context" in generated, generated
        assert "TODO" not in generated, generated
        tech_design_at = generated.find("## Tech Design")
        capability_map_at = generated.find("## Capability Map")
        assert tech_design_at != -1, generated
        assert capability_map_at != -1, generated
        assert tech_design_at < capability_map_at, generated

        manual_after = (root / "apps/tool/src/manual.rs").read_text(encoding="utf-8")
        assert manual_after == _MANUAL_RS, manual_after

        status = _git(root, "status", "--porcelain").stdout
        assert status == "", status
        log_message = _git(root, "log", "-1", "--pretty=%B").stdout
        assert "Lifecycle-Stage: Cb-Force-Regen" in log_message, log_message

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
