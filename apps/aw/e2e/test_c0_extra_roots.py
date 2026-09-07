"""`C0 scope`'s widening by a project's own `impl-roots` declaration.

`leg.c0_scope` reads that declaration from `apps/<project>/Cargo.toml` **as of
HEAD** (`leg.impl_extra_roots`), never the working tree, so each fixture here
is a real git repository with a committed manifest and `c0_scope` is called
directly with a hand-built `dirty` list -- there is no need for the fixture's
working tree to actually be dirty, because `c0_scope` only ever consumes the
list it is handed.
"""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

from aw.scripts import leg

DEMO_ROOT_WITH_DECLARATION = '''[package]
name = "demo"
version = "0.0.0"
autotests = false

[package.metadata.aw]
impl-roots = ["mambalibs/arraykit/pyo3"]
impl-repo-paths = ["Cargo.toml", "Cargo.lock"]
'''

DEMO_ROOT_WITHOUT_DECLARATION = '''[package]
name = "demo"
version = "0.0.0"
autotests = false
'''

DIRTY_WITH_DECLARED_ROOT = [
    "apps/demo/mambalibs/arraykit/pyo3/Cargo.toml",
    "apps/demo/mambalibs/arraykit/pyo3/pyproject.toml",
    "apps/demo/mambalibs/arraykit/pyo3/src/lib.rs",
    "apps/demo/mambalibs/arraykit/pyo3/src/tests.rs",
    "Cargo.toml",
    "Cargo.lock",
]


def _git_repo(tmp_path: Path, manifest: str) -> Path:
    """A committed repo with `apps/demo/Cargo.toml` carrying `manifest`."""
    project = tmp_path / "apps" / "demo"
    (project / "src").mkdir(parents=True)
    (project / "src" / ".gitkeep").write_text("", encoding="utf-8")
    (project / "Cargo.toml").write_text(manifest, encoding="utf-8")

    env = {
        **os.environ,
        "GIT_AUTHOR_NAME": "test",
        "GIT_AUTHOR_EMAIL": "test@example.com",
        "GIT_COMMITTER_NAME": "test",
        "GIT_COMMITTER_EMAIL": "test@example.com",
    }
    subprocess.run([*leg.GIT, "init", "-q"], cwd=tmp_path, check=True, env=env)
    subprocess.run([*leg.GIT, "add", "-A"], cwd=tmp_path, check=True, env=env)
    subprocess.run(
        [*leg.GIT, "commit", "-q", "-m", "initial"], cwd=tmp_path, check=True, env=env
    )
    return tmp_path


def test_declared_root_and_repo_paths_pass_with_the_colocated_test(
    tmp_path: Path,
) -> None:
    repo = _git_repo(tmp_path, DEMO_ROOT_WITH_DECLARATION)
    chk = leg.Check()

    leg.c0_scope(
        chk, repo, repo / "apps" / "demo" / "src", DIRTY_WITH_DECLARED_ROOT, "impl"
    )

    assert len(chk.rows) == 1
    status, name, detail = chk.rows[0]
    assert status == "PASS"
    assert name == "C0 scope"
    assert "1 test file(s)" in detail


def test_an_undeclared_sibling_path_fails_and_is_named(tmp_path: Path) -> None:
    repo = _git_repo(tmp_path, DEMO_ROOT_WITH_DECLARATION)
    chk = leg.Check()
    dirty = [*DIRTY_WITH_DECLARED_ROOT, "apps/demo/mambalibs/other/file.rs"]

    leg.c0_scope(chk, repo, repo / "apps" / "demo" / "src", dirty, "impl")

    assert len(chk.rows) == 1
    status, name, detail = chk.rows[0]
    assert status == "FAIL"
    assert name == "C0 scope"
    assert "apps/demo/mambalibs/other/file.rs" in detail


def test_the_same_dirty_set_with_no_declaration_fails_as_it_does_today(
    tmp_path: Path,
) -> None:
    repo = _git_repo(tmp_path, DEMO_ROOT_WITHOUT_DECLARATION)
    chk = leg.Check()

    leg.c0_scope(
        chk, repo, repo / "apps" / "demo" / "src", DIRTY_WITH_DECLARED_ROOT, "impl"
    )

    assert len(chk.rows) == 1
    status, name, detail = chk.rows[0]
    assert status == "FAIL"
    assert name == "C0 scope"
    assert "changed outside apps/demo/src/:" in detail
