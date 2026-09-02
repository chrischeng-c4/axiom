"""Focused fixtures for the Cargo-backed E2E inventory."""

from __future__ import annotations

from pathlib import Path

from aw.scripts import e2e


def write_manifest(root: Path, required_features: str = "") -> Path:
    project = root / "apps" / "demo"
    project.mkdir(parents=True)
    (project / "Cargo.toml").write_text(
        f'''[package]
name = "demo"
version = "0.0.0"
autotests = false

[[test]]
name = "plain"
path = "e2e/plain.rs"

[[test]]
name = "gated"
path = "e2e/gated.rs"
required-features = [{required_features}]
''',
        encoding="utf-8",
    )
    return project / "e2e"


def test_inventory_runs_each_target_with_its_required_features(tmp_path: Path) -> None:
    inventory = e2e.E2eInventory(
        write_manifest(tmp_path, '"operator", "delegated-auth"')
    )

    assert inventory.problem == ""
    assert inventory.cases["plain"]["command"] == "cargo test -p demo --test plain"
    assert inventory.cases["gated"]["command"] == (
        "cargo test -p demo --features operator,delegated-auth --test gated"
    )


def test_inventory_rejects_a_non_list_required_features_value(tmp_path: Path) -> None:
    e2e_root = write_manifest(tmp_path, '"operator"')
    manifest = e2e_root.parent / "Cargo.toml"
    manifest.write_text(
        manifest.read_text(encoding="utf-8").replace(
            'required-features = ["operator"]', 'required-features = "operator"'
        ),
        encoding="utf-8",
    )

    inventory = e2e.E2eInventory(e2e_root)

    assert "required-features" in inventory.problem
