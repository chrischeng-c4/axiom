"""Black-box contract for the XML HANDWRITE marker fill-queue lifecycle (#3310).

Drives the real `aw cb gen` -> `aw cb fill` pipeline over a TD spec with two
`impl_mode: hand-written` Changes entries that both anchor pre-existing Rust
functions under the same `section:`, proving a genuinely `aw cb gen`
scaffolded XML HANDWRITE marker (tracker="pending-tracker") stays queued for
`aw cb fill` despite wrapping non-empty pre-existing source, and that two
markers whose gap ids would otherwise collide get disambiguated within the
same active TD scope.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, git_commit_fixture, project_fixture, run_aw

CASE_ID = "existing-project-standardization-xml-handwrite-marker-fill-queue-lifecycle"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "xml-handwrite-marker-fill-queue-lifecycle"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-xml-handwrite-marker-fill-queue-lifecycle"
)
ASSERTIONS = (
    "aw cb gen, run against a TD spec with an impl_mode: hand-written + "
    "anchor Changes entry targeting a pre-existing, non-empty Rust "
    "function, scaffolds a real XML HANDWRITE marker carrying "
    "tracker=\"pending-tracker\" around that function's own existing body "
    "(rather than discarding it), and the marker still shows up in aw cb "
    "fill's own queued-marker enumeration afterward, proving the "
    "gen-to-fill handoff genuinely persists a still-pending marker instead "
    "of silently resolving or dropping it",
    "two such hand-written Changes entries that share one `section:` -- so "
    "aw cb gen would otherwise scaffold both markers under the identical "
    "gap id -- come back from aw cb fill's queue as two distinct marker "
    "ids that both still carry the shared gap prefix, proving aw cb fill "
    "disambiguates colliding marker ids within one active TD scope instead "
    "of silently merging or dropping one of them",
)

_SPEC_PATH = "tech-design/logic/existing-project-standardization-fixture.md"
_SPEC_TEMPLATE = """---
id: existing-project-standardization-fixture
fill_sections: [logic, changes]
---

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: src/gate_a.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: handler_a
  - path: src/gate_b.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: handler_b
```

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
  start([fixture]) --> done[done]
```
"""


def _change_body() -> str:
    return (
        "## Problem\n\nFixture WI driving the real aw cb gen -> aw cb fill "
        "hand-written HANDWRITE marker queue and disambiguation path over "
        "pre-existing source.\n"
    )


def _workspace_slug(root: Path) -> str:
    resolved = str(root.resolve())
    collapsed = re.sub(r"[^a-zA-Z0-9]+", "-", resolved)
    return collapsed.strip("-").lower()


def _issue_path(root: Path, slug: str, state: str) -> Path:
    return Path("/tmp/aw/workspaces") / _workspace_slug(root) / "issues" / state / f"{slug}.md"


def _relabel_phase(root: Path, slug: str, new_phase: str) -> None:
    path = _issue_path(root, slug, "open")
    assert path.is_file(), path
    original = path.read_text(encoding="utf-8")

    field_pattern = re.compile(r"(?m)^phase: .*$")
    assert field_pattern.search(original), original
    updated = field_pattern.sub(f"phase: {new_phase}", original, count=1)
    assert updated != original, original

    label_pattern = re.compile(r"(?m)^- phase:.*$")
    if label_pattern.search(updated):
        updated = label_pattern.sub(f"- phase:{new_phase}", updated, count=1)

    assert f"phase: {new_phase}" in updated, updated
    path.write_text(updated, encoding="utf-8")


_TD_PYPROJECT = """[project]
name = "demo-tech-design"
version = "0.1.0"
requires-python = ">=3.11"
"""

_TD_UV_LOCK = """version = 1
revision = 3
requires-python = ">=3.11"

[[package]]
name = "demo-tech-design"
version = "0.1.0"
source = { virtual = "." }
"""


def _fill_python_td_module(module_path: Path, slug: str) -> None:
    """Replace the `aw td create` AW_TD_FILL scaffold with a trivial real body.

    `aw td create --apply` only requires the module to (a) drop the fill
    marker, (b) keep its `__aw_work_item__`/`__aw_artifact_id__` bindings,
    and (c) compile with at least one declaration under the syntax-only
    Python TD compiler -- it never executes this module. The real Changes
    entries this case exercises live entirely in the separate Markdown
    `--spec-path` file driven through `aw cb gen`/`aw cb fill` below; this
    module only satisfies PythonV1's WI-to-TD-module binding requirement.
    """

    original = module_path.read_text(encoding="utf-8")
    assert "AW_TD_FILL" in original, original
    updated = original.replace(
        '    # AW_TD_FILL: replace this marker with executable Python TD declarations.\n'
        '    return "pending"\n',
        f'    return "bound to {slug}"\n',
    )
    assert "AW_TD_FILL" not in updated, updated
    assert updated != original, original
    module_path.write_text(updated, encoding="utf-8")


def verify() -> list[str]:
    with project_fixture() as root:
        src_dir = root / "src"
        src_dir.mkdir(parents=True, exist_ok=True)
        (src_dir / "gate_a.rs").write_text("pub fn handler_a() {\n    1;\n}\n", encoding="utf-8")
        (src_dir / "gate_b.rs").write_text("pub fn handler_b() {\n    2;\n}\n", encoding="utf-8")

        td_dir = root / "tech-design"
        td_dir.mkdir(parents=True, exist_ok=True)
        (td_dir / "pyproject.toml").write_text(_TD_PYPROJECT, encoding="utf-8")
        (td_dir / "uv.lock").write_text(_TD_UV_LOCK, encoding="utf-8")

        spec_abs = root / _SPEC_PATH
        spec_abs.parent.mkdir(parents=True, exist_ok=True)
        spec_abs.write_text(_SPEC_TEMPLATE, encoding="utf-8")

        created = create(
            root,
            "Fixture: xml handwrite marker fill queue",
            "change",
            "--body",
            _change_body(),
        )
        slug = created["slug"]
        validated = final_json(run_aw(root, "wi", "validate", slug))
        assert validated["passed"] is True, validated

        git_commit_fixture(root)

        # Jump the WI straight to `td_inited` -- the phase `aw td create`
        # (brief) requires -- instead of driving the full ec-checked /
        # ec-reviewed lead-up sequence, which is out of this case's scope.
        # The local-backend issue store lives under /tmp/aw/workspaces/**,
        # entirely outside `root`'s own git tree, so this relabel has no
        # git-tree diff of its own to commit.
        _relabel_phase(root, slug, "td_inited")

        brief = final_json(run_aw(root, "td", "create", slug))
        module_rel = brief["artifact"]["source_path"]
        module_abs = root / module_rel
        assert module_abs.is_file(), (module_rel, brief)

        _fill_python_td_module(module_abs, slug)
        git_commit_fixture(root, "fixture: fill td module")

        run_aw(
            root,
            "td",
            "create",
            slug,
            "--apply",
            "--spec-path",
            module_rel,
            "--project",
            "demo",
        )

        run_aw(root, "td", "lock", "--project", "demo")
        git_commit_fixture(root, "fixture: td lock")

        run_aw(root, "cb", "gen", slug, "--spec-path", _SPEC_PATH)

        gate_a = (src_dir / "gate_a.rs").read_text(encoding="utf-8")
        gate_b = (src_dir / "gate_b.rs").read_text(encoding="utf-8")
        for path, body, anchor in (
            (src_dir / "gate_a.rs", gate_a, "pub fn handler_a()"),
            (src_dir / "gate_b.rs", gate_b, "pub fn handler_b()"),
        ):
            assert "<HANDWRITE" in body, (path, body)
            assert 'tracker="pending-tracker"' in body, (path, body)
            assert anchor in body, (path, body)

        completed = run_aw(root, "cb", "fill", slug, "--spec-path", _SPEC_PATH)
        envelope = final_json(completed)
        marker_list = envelope["invoke"]["args"]["marker_list"]
        assert len(marker_list) == 2, envelope
        ids = [marker["id"] for marker in marker_list]
        assert len(set(ids)) == 2, ids
        assert all(marker_id.startswith("missing-generator:logic--") for marker_id in ids), ids
        source_paths = {marker["source_path"] for marker in marker_list}
        assert source_paths == {"src/gate_a.rs", "src/gate_b.rs"}, marker_list

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
