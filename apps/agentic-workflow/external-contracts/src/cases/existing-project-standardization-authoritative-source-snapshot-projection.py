"""Black-box contract for `aw cb gen-source`'s authoritative source snapshot (#3309)."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import run_aw

CASE_ID = "existing-project-standardization-authoritative-source-snapshot-projection"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "authoritative-source-snapshot-projection"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-authoritative-source-snapshot-projection"
)
ASSERTIONS = (
    "aw cb gen-source --spec <spec> --target <target>, run against a target "
    "file whose live content diverges from a TD Source section's embedded "
    "<!-- source-snapshot --> legacy Rust fence, overwrites the target with "
    "the embedded snapshot's content exactly (not a strip-handwrite "
    "reconstruction of the target's own prior content), reports exactly one "
    "block updated and wrote=true in both its terminal aw.cli.v1 JSON envelope "
    "and its human-readable summary line, and leaves an untouched sibling file "
    "byte-for-byte unchanged, proving the embedded snapshot -- not the live "
    "target -- is the authoritative projection source",
    "an identical second aw cb gen-source invocation against the "
    "now-already-projected target is a real no-op: zero blocks updated, zero "
    "files created, wrote=false, while still exiting 0 with "
    "completion.workflow_complete=true, proving snapshot projection is "
    "idempotent rather than rewriting the target on every pass",
)

_SIBLING_RS = "pub fn untouched() {}\n"
_OLD_TARGET_RS = (
    "// SPEC-MANAGED: tech-design/direct.md#source\n"
    "// CODEGEN-BEGIN\n"
    'pub const SNAPSHOT_VALUE: &str = "before";\n'
    "// CODEGEN-END\n"
)
_PROJECTED_TARGET_RS = (
    "// SPEC-MANAGED: tech-design/direct.md#source\n"
    "// CODEGEN-BEGIN\n"
    'pub const SNAPSHOT_VALUE: &str = "after";\n'
    "// CODEGEN-END\n"
)

_SPEC = (
    "---\n"
    "id: legacy-snapshot-test\n"
    "fill_sections: [overview, changes]\n"
    "---\n\n"
    "# Legacy Snapshot Test\n\n"
    "## Overview\n"
    "<!-- type: overview lang: markdown -->\n\n"
    "Exact legacy snapshot projection fixture.\n\n"
    "## Source\n"
    "<!-- type: source lang: rust -->\n"
    "<!-- source-from-target: strip-handwrite -->\n\n"
    "<!-- source-snapshot: path=src/lib.rs -->\n"
    "```rust\n" + _PROJECTED_TARGET_RS + "```\n\n"
    "## Changes\n"
    "<!-- type: changes lang: yaml -->\n\n"
    "```yaml\n"
    "changes:\n"
    "  - path: src/lib.rs\n"
    "    action: create\n"
    "    section: source\n"
    "    impl_mode: codegen\n"
    "  - path: src/lib.rs\n"
    "    action: modify\n"
    "    section: source\n"
    "    impl_mode: codegen\n"
    "```\n"
)


def _git(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(
        ["git", *args], cwd=root, capture_output=True, text=True, check=False
    )
    if completed.returncode != 0:
        raise AssertionError(f"git {' '.join(args)} failed: {completed.stderr}")
    return completed


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-ec-gen-source-") as raw_root:
        root = Path(raw_root)
        (root / "aw.toml").write_text(
            '[[projects]]\nname = "demo"\npath = "."\nlabel = "app:demo"\n',
            encoding="utf-8",
        )
        (root / "tech-design").mkdir(parents=True)
        (root / "src").mkdir(parents=True)
        (root / "tech-design" / "direct.md").write_text(_SPEC, encoding="utf-8")
        (root / "src" / "lib.rs").write_text(_OLD_TARGET_RS, encoding="utf-8")
        (root / "src" / "sibling.rs").write_text(_SIBLING_RS, encoding="utf-8")
        _git(root, "init")
        _git(root, "config", "user.email", "fixture@example.com")
        _git(root, "config", "user.name", "Fixture")
        _git(root, "add", "-A")
        _git(root, "commit", "-m", "fixture")

        first = run_aw(
            root, "cb", "gen-source", "--spec", "tech-design/direct.md", "--target", "src/lib.rs"
        )
        assert '"blocks_updated":1' in first.stdout, first.stdout
        assert '"files_created":0' in first.stdout, first.stdout
        assert '"wrote_files":true' in first.stdout, first.stdout
        assert '"workflow_complete":true' in first.stdout, first.stdout
        assert "1 block(s) updated, 0 file(s) created, wrote=true" in first.stderr, first.stderr
        assert (root / "src" / "lib.rs").read_text(encoding="utf-8") == _PROJECTED_TARGET_RS
        assert (root / "src" / "sibling.rs").read_text(encoding="utf-8") == _SIBLING_RS

        second = run_aw(
            root, "cb", "gen-source", "--spec", "tech-design/direct.md", "--target", "src/lib.rs"
        )
        assert '"blocks_updated":0' in second.stdout, second.stdout
        assert '"files_created":0' in second.stdout, second.stdout
        assert '"wrote_files":false' in second.stdout, second.stdout
        assert '"workflow_complete":true' in second.stdout, second.stdout
        assert "0 block(s) updated, 0 file(s) created, wrote=false" in second.stderr, second.stderr

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
