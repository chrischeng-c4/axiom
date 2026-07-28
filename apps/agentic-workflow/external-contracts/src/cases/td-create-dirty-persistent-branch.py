"""Black-box TD activation contract for dirty persistent project branches."""

from __future__ import annotations

import subprocess
from pathlib import Path

from migration_clusters.work_item_planning import BOUNDED_BODY
from wi_contract_fixture import create, final_json, project_fixture, run_aw


CASE_ID = "td-create-dirty-persistent-branch"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-create-dirty-persistent-branch"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "python3 apps/agentic-workflow/external-contracts/src/runner.py "
    "--case td-create-dirty-persistent-branch"
)
ASSERTIONS = (
    "dirty persistent branch initializes TD in place",
    "unrelated modified deleted and untracked paths remain byte-identical",
    "successful TD initialization advances HEAD with exactly the emitted tracked TD source",
    "dirty main fails before branch activation or TD source creation",
)


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


def verify() -> list[str]:
    with project_fixture() as persistent:
        _initialize_repository(persistent)
        _git(persistent, "checkout", "-qb", "project-demo")
        slug = _create_open_change(persistent, "Dirty persistent branch fixture")

        (persistent / "tracked.txt").write_text("user edit\n", encoding="utf-8")
        (persistent / "deleted.txt").unlink()
        (persistent / "untracked.txt").write_text(
            "user scratch\n", encoding="utf-8"
        )
        before = _git(persistent, "status", "--porcelain")
        before_head = _git(persistent, "rev-parse", "HEAD").strip()

        payload = final_json(
            run_aw(persistent, "td", "create", slug, "--project", "demo")
        )
        source_path = payload["artifact"]["source_path"]
        source = persistent / source_path
        after_head = _git(persistent, "rev-parse", "HEAD").strip()

        assert payload["action"] == "dispatch"
        assert payload["target"]["branch"] == "project-demo"
        assert _git(persistent, "branch", "--show-current").strip() == "project-demo"
        assert _git(persistent, "status", "--porcelain") == before
        assert after_head != before_head
        assert source.is_file()
        assert f'__aw_work_item__ = "{slug}"' in source.read_text(encoding="utf-8")
        assert _git(persistent, "ls-files", "--error-unmatch", source_path).strip() == source_path
        committed_paths = sorted(
            path
            for path in _git(
                persistent,
                "diff",
                "--name-only",
                before_head,
                after_head,
            ).splitlines()
            if path
        )
        assert committed_paths == [source_path]
        assert (persistent / "tracked.txt").read_text(encoding="utf-8") == "user edit\n"
        assert (
            persistent / "untracked.txt"
        ).read_text(encoding="utf-8") == "user scratch\n"
        assert not (persistent / "deleted.txt").exists()

    with project_fixture() as main:
        _initialize_repository(main)
        slug = _create_open_change(main, "Dirty main branch fixture")
        (main / "tracked.txt").write_text("dirty main\n", encoding="utf-8")
        (main / "untracked.txt").write_text(
            "dirty main scratch\n", encoding="utf-8"
        )
        before = _git(main, "status", "--porcelain")

        failed = run_aw(
            main,
            "td",
            "create",
            slug,
            "--project",
            "demo",
            expect_success=False,
        )

        assert "requires a clean tree" in f"{failed.stdout}\n{failed.stderr}"
        assert _git(main, "branch", "--show-current").strip() == "main"
        assert _git(main, "status", "--porcelain") == before
        assert not (main / "tech-design").exists()
        assert (
            subprocess.run(
                ["git", "show-ref", "--verify", "--quiet", f"refs/heads/td-{slug}"],
                cwd=main,
                check=False,
            ).returncode
            != 0
        )

    return list(ASSERTIONS)
