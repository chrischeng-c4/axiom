"""Maintenance Change point path normalization contract."""

from __future__ import annotations

import pytest

from aw.scripts import maint


@pytest.mark.parametrize(
    ("token", "expected"),
    [
        (".github/workflows/ci.yml", ".github/workflows/ci.yml"),
        (".cargo/config.toml", ".cargo/config.toml"),
        ("apps/demo/README.md", "apps/demo/README.md"),
        ("scripts/check.py", "scripts/check.py"),
        ("README.md", "apps/aw/README.md"),
        ("src/aw/scripts/maint.py", "apps/aw/src/aw/scripts/maint.py"),
        ("(`.github/workflows/ci.yml`,)", ".github/workflows/ci.yml"),
        (".github/workflows/ci.yml:12,", ".github/workflows/ci.yml"),
    ],
)
def test_normalise_declared_preserves_repository_root_paths(
    token: str, expected: str
) -> None:
    assert maint._normalise_declared(token, "apps/aw") == expected


@pytest.mark.parametrize("token", ["/tmp/outside", "../outside", "src/../outside"])
def test_normalise_declared_refuses_absolute_or_parent_paths(token: str) -> None:
    with pytest.raises(maint.MaintError):
        maint._normalise_declared(token, "apps/aw")


@pytest.mark.parametrize(
    "path",
    [
        "apps/tape/Dockerfile",
        "apps/tape/Dockerfile.release",
        "apps/tape/Dockerfile.debug",
    ],
)
def test_chore_path_accepts_dockerfile_variants(path: str) -> None:
    assert maint._is_chore_path(path)


@pytest.mark.parametrize(
    "path",
    [
        "apps/tape/Dockerfilex",
        "apps/tape/src/product.py",
    ],
)
def test_chore_path_rejects_dockerfile_near_misses_and_product_files(path: str) -> None:
    assert not maint._is_chore_path(path)
