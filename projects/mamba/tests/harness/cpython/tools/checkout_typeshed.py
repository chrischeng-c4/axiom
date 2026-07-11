#!/usr/bin/env python3.12
"""Create or verify the exact typeshed checkout pinned by mamba."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from typeshed_lock import (  # noqa: E402
    DEFAULT_LOCK_PATH,
    TypeshedLock,
    TypeshedLockError,
    load_typeshed_lock,
    verify_typeshed_stdlib,
)


MAMBA_DIR = Path(__file__).resolve().parents[4]
DEFAULT_TARGET = MAMBA_DIR / "vendor" / "typeshed"


def _git(*args: str, cwd: Path | None = None) -> None:
    subprocess.run(["git", *args], cwd=cwd, check=True)


def _git_output(*args: str, cwd: Path) -> str:
    return subprocess.run(
        ["git", *args],
        cwd=cwd,
        check=True,
        text=True,
        capture_output=True,
    ).stdout.strip()


def verify_git_checkout(target: Path, lock: TypeshedLock) -> None:
    if not (target / ".git").exists():
        raise TypeshedLockError(f"existing target is not a git checkout: {target}")
    try:
        top_level = _git_output("rev-parse", "--show-toplevel", cwd=target)
        head = _git_output("rev-parse", "HEAD", cwd=target)
        origin = _git_output("config", "--get", "remote.origin.url", cwd=target)
        dirty = _git_output("status", "--porcelain", "--untracked-files=all", cwd=target)
    except subprocess.CalledProcessError as error:
        raise TypeshedLockError(f"existing target is not a git checkout: {target}") from error
    if Path(top_level).resolve() != target.resolve():
        raise TypeshedLockError(
            f"typeshed checkout root expected={target.resolve()} actual={top_level}"
        )
    if head != lock.revision:
        raise TypeshedLockError(
            f"typeshed checkout revision expected={lock.revision} actual={head}"
        )
    if origin != lock.repository:
        raise TypeshedLockError(
            f"typeshed checkout origin expected={lock.repository} actual={origin}"
        )
    if dirty:
        raise TypeshedLockError(
            f"typeshed checkout has local changes; refusing to replace them: {dirty}"
        )
    verify_typeshed_stdlib(target / "stdlib", lock=lock)


def checkout_typeshed(target: Path, lock: TypeshedLock) -> None:
    if target.exists():
        verify_git_checkout(target, lock)
        return

    target.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(
        tempfile.mkdtemp(prefix=".typeshed-checkout-", dir=target.parent)
    )
    try:
        _git("init", str(temporary))
        _git("remote", "add", "origin", lock.repository, cwd=temporary)
        _git("fetch", "--depth=1", "origin", lock.revision, cwd=temporary)
        _git("checkout", "--detach", "FETCH_HEAD", cwd=temporary)
        verify_git_checkout(temporary, lock)
        os.replace(temporary, target)
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", type=Path, default=DEFAULT_TARGET)
    parser.add_argument(
        "--verify-only",
        action="store_true",
        help="verify the existing checkout revision, origin, cleanliness, and stdlib",
    )
    args = parser.parse_args(argv)

    try:
        lock = load_typeshed_lock(DEFAULT_LOCK_PATH)
        target = args.target.resolve()
        if args.verify_only:
            verify_git_checkout(target, lock)
        else:
            checkout_typeshed(target, lock)
    except (OSError, subprocess.CalledProcessError, TypeshedLockError) as error:
        print(f"typeshed checkout failed: {error}", file=sys.stderr)
        return 2

    print(
        f"typeshed ready: target={target} revision={lock.revision} "
        f"stdlib_pyi_count={lock.stdlib_pyi_count} "
        f"stdlib_pyi_sha256={lock.stdlib_pyi_sha256}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
