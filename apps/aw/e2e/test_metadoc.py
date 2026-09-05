"""Focused fixtures for META-doc checks."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

from aw.scripts import metadoc


PROJECT = "apps/demo"


def ready_project(tmp_path: Path) -> Path:
    root = tmp_path / PROJECT
    (root / "docs").mkdir(parents=True)
    (root / "README.md").write_text("# Demo\n", encoding="utf-8")
    (root / "STATUS.md").write_text("# Status\n", encoding="utf-8")
    (root / "ROADMAP.md").write_text("# Roadmap\n\n- ID: `future`\n",
                                      encoding="utf-8")
    validator = tmp_path / metadoc.VALIDATOR
    validator.parent.mkdir(parents=True)
    validator.write_text(
        "import json\nprint(json.dumps({'reports': [{'ok': True}]}))\n",
        encoding="utf-8",
    )
    return root


def p8_findings(tmp_path: Path, monkeypatch, changed: list[str]) -> list[metadoc.Finding]:
    monkeypatch.setattr(metadoc.leg, "dirty_set", lambda _repo: changed)
    monkeypatch.setattr(metadoc, "pinned_launcher", lambda: [sys.executable])
    findings, _population = metadoc.collect(tmp_path, PROJECT)
    return [finding for finding in findings if finding.rule == "P8"]


def test_p8_ignores_changed_reference_in_unindexed_docs_directory(
        tmp_path: Path, monkeypatch) -> None:
    root = ready_project(tmp_path)
    reference = root / "docs/reference/notes.md"
    reference.parent.mkdir(parents=True)
    reference.write_text("# Notes\n\nReference prose.\n", encoding="utf-8")
    path = f"{PROJECT}/docs/reference/notes.md"
    monkeypatch.setattr(metadoc.leg, "dirty_set", lambda _repo: [path])
    monkeypatch.setattr(metadoc, "pinned_launcher", lambda: [sys.executable])

    findings, population = metadoc.collect(tmp_path, PROJECT)

    assert not findings
    assert population["areas"] == 0
    assert population["reference"] == 1


def test_p8_refuses_an_indexed_area_that_disagrees_with_its_index(
        tmp_path: Path, monkeypatch) -> None:
    root = ready_project(tmp_path)
    folder = root / "docs/product"
    folder.mkdir()
    (folder / "README.md").write_text(
        "## Section index\n\n| Section | File |\n|---|---|\n| Other | area.md |\n",
        encoding="utf-8",
    )
    (folder / "area.md").write_text(
        "## Future\n\n"
        "- Problem: problem.\n- Who: people.\n- Promise: promise.\n"
        "- Non-goals: none.\n- Open: none.\n- Neighbours: none.\n"
        "- Outcome: `future`. Tracking: not assigned.\n\n"
        "## Non-goals in this area\n",
        encoding="utf-8",
    )

    findings = p8_findings(tmp_path, monkeypatch, [f"{PROJECT}/docs/product/area.md"])

    assert findings


def test_p8_refuses_a_deleted_directory_index(tmp_path: Path, monkeypatch) -> None:
    root = ready_project(tmp_path)
    (root / "docs/product").mkdir()

    findings = p8_findings(tmp_path, monkeypatch,
                           [f"{PROJECT}/docs/product/README.md"])

    assert len(findings) == 1
    assert findings[0].path == f"{PROJECT}/docs/product/README.md"


def test_only_internal_release_apply_may_add_a_milestone_binding(
        tmp_path: Path, monkeypatch) -> None:
    root = ready_project(tmp_path)
    folder = root / "docs/product"
    folder.mkdir()
    (folder / "README.md").write_text(
        "## Section index\n\n| Section | File |\n|---|---|\n| Future | area.md |\n",
        encoding="utf-8",
    )
    area = folder / "area.md"
    area.write_text(
        "## Future (Milestone #7)\n\n"
        "- Problem: problem.\n- Who: people.\n- Promise: promise.\n"
        "- Non-goals: none.\n- Open: none.\n- Neighbours: none.\n"
        "- Outcome: `future`. Tracking: not assigned.\n\n"
        "## Non-goals in this area\n",
        encoding="utf-8",
    )
    path = f"{PROJECT}/docs/product/area.md"
    before = area.read_text(encoding="utf-8").replace(" (Milestone #7)", "")
    monkeypatch.setattr(metadoc.leg, "dirty_set", lambda _repo: [path])
    monkeypatch.setattr(metadoc, "git_show", lambda _repo, _path: before)
    monkeypatch.setattr(metadoc, "pinned_launcher", lambda: [sys.executable])

    public, _population = metadoc.collect(tmp_path, PROJECT)
    approved, _population = metadoc.collect(
        tmp_path, PROJECT, release_milestone_number=7,
    )
    wrong_number, _population = metadoc.collect(
        tmp_path, PROJECT, release_milestone_number=8,
    )

    assert [finding for finding in public if finding.rule == "P4"]
    assert not [finding for finding in approved if finding.rule == "P4"]
    assert [finding for finding in wrong_number if finding.rule == "P4"]


@pytest.mark.parametrize("option", [
    "--allow-release-binding", "--release-milestone-number",
])
def test_public_metadoc_parser_exposes_no_binding_bypass(option: str) -> None:
    with pytest.raises(SystemExit) as error:
        metadoc.main(["check", PROJECT, option, "7"])
    assert error.value.code == 2
