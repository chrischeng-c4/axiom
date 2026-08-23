#!/usr/bin/env python3
"""Run the AGY dispatcher outside the caller repository's UV project."""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path


DISPATCHER = Path(__file__).with_name("agy_dispatch.py")


def runtime_environment(source: dict[str, str] | None = None) -> dict[str, str]:
    """Return an environment whose UV virtualenv cannot be a repository path."""
    environment = dict(os.environ if source is None else source)
    runtime = Path(
        environment.get(
            "AGY_DISPATCH_RUNTIME",
            str(Path(tempfile.gettempdir()) / "agy-dispatch-runtime"),
        )
    ).expanduser()
    if not runtime.is_absolute():
        raise SystemExit("AGY_DISPATCH_RUNTIME must be an absolute temporary path")
    runtime = runtime.resolve()
    temporary_root = Path(tempfile.gettempdir()).resolve()
    if runtime != temporary_root and not runtime.is_relative_to(temporary_root):
        raise SystemExit("AGY_DISPATCH_RUNTIME must resolve under the system temp root")
    runtime.mkdir(parents=True, exist_ok=True)
    environment["UV_PROJECT_ENVIRONMENT"] = str(runtime)
    return environment


def command(arguments: list[str]) -> list[str]:
    return [
        "uv",
        "run",
        "--isolated",
        "--no-project",
        "--python",
        "3.13",
        str(DISPATCHER),
        *arguments,
    ]


def main(arguments: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if arguments is None else arguments)
    return subprocess.run(command(args), env=runtime_environment(), check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
