"""Black-box existing-TD-workspace activation contract."""

from __future__ import annotations

import subprocess
from pathlib import Path

from migration_clusters.work_item_planning import BOUNDED_BODY
from wi_contract_fixture import (
    create,
    final_json,
    project_fixture,
    run_aw,
    write_python_artifact_unit_test,
)


CASE_ID = "td-existing-workspace-dirty-persistent-branch"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "dirty-persistent-branch-existing-td-activation"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case td-existing-workspace-dirty-persistent-branch"
)
ASSERTIONS = (
    "public TD apply rejects malformed Python then advances with a semantic digest",
    "public CB generation stays on a dirty persistent branch and commits only its generated target",
    "existing-workspace activation preserves modified deleted and untracked paths while rejecting staged paths",
    "dirty main fails before switching to an existing TD branch",
    "public CB generation on clean main without a TD branch retains the workspace-not-found remediation",
)

GENERATION_PLAN = r'''

GENERATION_PLAN = r"""## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  Widget:
    type: object
    properties:
      name: { type: string }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: src/generated.rs
    action: create
    section: schema
    impl_mode: codegen
```
"""
'''


def _git(root: Path, *args: str) -> str:
    completed = subprocess.run(
        ["git", *args],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise AssertionError(
            f"git {' '.join(args)} failed:\n"
            f"stdout={completed.stdout}\nstderr={completed.stderr}"
        )
    return completed.stdout


def _initialize_repository(root: Path) -> None:
    _git(root, "init")
    _git(root, "config", "user.email", "fixture@example.com")
    _git(root, "config", "user.name", "Fixture")
    (root / "tracked.txt").write_text("baseline\n", encoding="utf-8")
    (root / "deleted.txt").write_text("keep in history\n", encoding="utf-8")
    _git(root, "add", "aw.toml", "tracked.txt", "deleted.txt")
    _git(root, "commit", "-m", "fixture baseline")


def _create_open_change(root: Path, title: str) -> str:
    created = create(root, title, "change", "--body", BOUNDED_BODY)
    slug = created["slug"]
    run_aw(root, "wi", "update", slug, "--state", "open")
    return slug


def _initialize_td(root: Path, slug: str) -> str:
    payload = final_json(run_aw(root, "td", "create", slug, "--project", "demo"))
    return payload["artifact"]["source_path"]


def _authored_source(scaffold: str) -> str:
    return (
        scaffold.replace(
            "    # AW_TD_FILL: replace this marker with executable Python TD declarations.",
            "    # Executable Python TD contract.",
        ).replace(
            '    return "pending"',
            '    return "aw.python-td-ir.v1"',
        )
        + GENERATION_PLAN
    )


def _branch_ref(root: Path, branch: str) -> str | None:
    completed = subprocess.run(
        ["git", "rev-parse", "--verify", f"refs/heads/{branch}"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    return completed.stdout.strip() if completed.returncode == 0 else None


def _without_status_path(status: str, path: str) -> str:
    return "".join(
        line
        for line in status.splitlines(keepends=True)
        if not line.rstrip().endswith(path)
    )


def _write_generation_spec(root: Path) -> str:
    spec_path = "tech-design/missing-workspace.md"
    spec = root / spec_path
    spec.parent.mkdir(parents=True, exist_ok=True)
    spec.write_text(
        """## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  Widget:
    type: object
    properties:
      name: { type: string }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: src/generated.rs
    action: create
    section: schema
    impl_mode: codegen
```
""",
        encoding="utf-8",
    )
    return spec_path


def verify() -> list[str]:
    with project_fixture() as persistent:
        _initialize_repository(persistent)
        _git(persistent, "checkout", "-qb", "project-demo")
        slug = _create_open_change(persistent, "Existing TD workspace fixture")
        source_path = _initialize_td(persistent, slug)
        source = persistent / source_path
        scaffold = source.read_text(encoding="utf-8")

        (persistent / "tracked.txt").write_text("user edit\n", encoding="utf-8")
        (persistent / "deleted.txt").unlink()
        (persistent / "untracked.txt").write_text(
            "user scratch\n", encoding="utf-8"
        )

        source.write_text(
            _authored_source(scaffold) + "\ndef broken(:\n",
            encoding="utf-8",
        )
        red_head = _git(persistent, "rev-parse", "HEAD").strip()
        red_status = _git(persistent, "status", "--porcelain")

        rejected = run_aw(
            persistent,
            "td",
            "create",
            slug,
            "--apply",
            "--spec-path",
            source_path,
            "--project",
            "demo",
            expect_success=False,
        )

        assert "Python TD diagnostic [syntax-error]" in rejected.stderr, rejected.stderr
        assert _git(persistent, "rev-parse", "HEAD").strip() == red_head
        assert _git(persistent, "status", "--porcelain") == red_status
        assert _git(persistent, "branch", "--show-current").strip() == "project-demo"

        source.write_text(_authored_source(scaffold), encoding="utf-8")
        before_head = _git(persistent, "rev-parse", "HEAD").strip()
        before_status = _git(persistent, "status", "--porcelain")

        applied = final_json(
            run_aw(
                persistent,
                "td",
                "create",
                slug,
                "--apply",
                "--spec-path",
                source_path,
                "--project",
                "demo",
            )
        )
        after_head = _git(persistent, "rev-parse", "HEAD").strip()

        assert applied["action"] == "dispatch"
        assert applied["artifact"]["source_path"] == source_path
        assert applied["artifact"]["semantic_digest"].startswith("sha256:")
        assert applied["next"]["command"] == (
            f"aw td check tech-design --project demo --wi {slug}"
        )
        assert _git(persistent, "branch", "--show-current").strip() == "project-demo"
        assert _git(persistent, "status", "--porcelain") == _without_status_path(
            before_status,
            source_path,
        )
        assert after_head != before_head
        assert sorted(
            _git(
                persistent,
                "diff",
                "--name-only",
                before_head,
                after_head,
            ).splitlines()
        ) == [source_path]
        assert _branch_ref(persistent, f"td-{slug}") is None
        assert (persistent / "tracked.txt").read_text(encoding="utf-8") == "user edit\n"
        assert (
            persistent / "untracked.txt"
        ).read_text(encoding="utf-8") == "user scratch\n"
        assert not (persistent / "deleted.txt").exists()

        # TD activation bootstraps the manifest and lock but leaves unit tests to
        # the author, and `aw td check` requires at least one. Committing it here
        # keeps the later generation snapshots clean.
        unit_test = write_python_artifact_unit_test(
            persistent / "tech-design", "existing_workspace"
        )
        _git(persistent, "add", str(unit_test.relative_to(persistent)))
        _git(persistent, "commit", "-m", "author fixture TD unit test")

        run_aw(
            persistent,
            "td",
            "check",
            "tech-design",
            "--project",
            "demo",
            "--wi",
            slug,
        )
        lock_path = "tech-design/td.lock"
        if (persistent / lock_path).is_file():
            _git(persistent, "add", lock_path)
            _git(persistent, "commit", "-m", "lock fixture TD")

        generation_head = _git(persistent, "rev-parse", "HEAD").strip()
        generation_status = _git(persistent, "status", "--porcelain")

        generated = final_json(
            run_aw(
                persistent,
                "cb",
                "gen",
                slug,
                "--spec-path",
                source_path,
            )
        )
        generated_head = _git(persistent, "rev-parse", "HEAD").strip()

        assert generated["action"] == "dispatch"
        assert generated["invoke"] == {
            "command": "aw cb check",
            "args": {"target": slug},
        }
        assert _git(persistent, "branch", "--show-current").strip() == "project-demo"
        after_generation_status = _git(persistent, "status", "--porcelain")
        assert after_generation_status == generation_status, (
            generation_status,
            after_generation_status,
        )
        assert generated_head != generation_head
        assert _git(
            persistent,
            "diff",
            "--name-only",
            generation_head,
            generated_head,
        ).splitlines() == ["src/generated.rs"]
        assert (persistent / "src/generated.rs").is_file()
        assert (persistent / "tracked.txt").read_text(encoding="utf-8") == "user edit\n"
        assert (
            persistent / "untracked.txt"
        ).read_text(encoding="utf-8") == "user scratch\n"
        assert not (persistent / "deleted.txt").exists()

    with project_fixture() as staged:
        _initialize_repository(staged)
        _git(staged, "checkout", "-qb", "project-demo")
        spec_path = _write_generation_spec(staged)
        _git(staged, "add", spec_path)
        _git(staged, "commit", "-m", "add generation plan")
        slug = _create_open_change(staged, "Staged path safety fixture")
        (staged / "staged.txt").write_text("staged user work\n", encoding="utf-8")
        _git(staged, "add", "staged.txt")
        before_head = _git(staged, "rev-parse", "HEAD").strip()
        before_status = _git(staged, "status", "--porcelain")

        failed = run_aw(
            staged,
            "cb",
            "gen",
            slug,
            "--spec-path",
            spec_path,
            expect_success=False,
        )

        assert "pre-existing staged paths" in failed.stderr
        assert _git(staged, "rev-parse", "HEAD").strip() == before_head
        assert _git(staged, "branch", "--show-current").strip() == "project-demo"
        assert _git(staged, "status", "--porcelain") == before_status
        assert (staged / "staged.txt").read_text(encoding="utf-8") == (
            "staged user work\n"
        )

    with project_fixture() as dirty_main:
        _initialize_repository(dirty_main)
        slug = _create_open_change(dirty_main, "Existing TD branch fixture")
        source_path = _initialize_td(dirty_main, slug)
        branch = f"td-{slug}"
        branch_ref = _branch_ref(dirty_main, branch)
        assert branch_ref is not None
        _git(dirty_main, "checkout", "main")
        (dirty_main / "tracked.txt").write_text("dirty main\n", encoding="utf-8")
        before_status = _git(dirty_main, "status", "--porcelain")

        failed = run_aw(
            dirty_main,
            "td",
            "create",
            slug,
            "--apply",
            "--spec-path",
            source_path,
            "--project",
            "demo",
            expect_success=False,
        )

        assert "requires a clean tree" in f"{failed.stdout}\n{failed.stderr}"
        assert _git(dirty_main, "branch", "--show-current").strip() == "main"
        assert _git(dirty_main, "status", "--porcelain") == before_status
        assert _branch_ref(dirty_main, branch) == branch_ref

    with project_fixture() as missing:
        _initialize_repository(missing)
        spec_path = _write_generation_spec(missing)
        _git(missing, "add", spec_path)
        _git(missing, "commit", "-m", "add generation plan")
        slug = _create_open_change(missing, "Missing TD branch fixture")
        branch = f"td-{slug}"
        assert _branch_ref(missing, branch) is None

        failed = run_aw(
            missing,
            "cb",
            "gen",
            slug,
            "--spec-path",
            spec_path,
            expect_success=False,
        )

        output = f"{failed.stdout}\n{failed.stderr}"
        assert "workspace not found" in output
        assert f"aw td create {slug}" in output
        assert _git(missing, "branch", "--show-current").strip() == "main"
        assert _branch_ref(missing, branch) is None

    return list(ASSERTIONS)
