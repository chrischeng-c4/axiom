"""Python EC implementation for shared CLI Report intake labels."""

from __future__ import annotations

import subprocess
from pathlib import Path


CASE_ID = "shared-cli-issue-report-intake"
REPOSITORY_ROOT = Path(__file__).resolve().parents[5]


def verify() -> list[str]:
    build = subprocess.run(
        ["cargo", "build", "-p", "cap", "--bin", "cap-full"],
        cwd=REPOSITORY_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert build.returncode == 0, build.stdout + build.stderr
    completed = subprocess.run(
        [
            str(REPOSITORY_ROOT / "target" / "debug" / "cap-full"),
            "issue",
            "create",
            "--title",
            "Shared CLI intake",
            "--dry-run",
            "Observed behavior",
        ],
        cwd=REPOSITORY_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert completed.returncode == 0, completed.stdout + completed.stderr
    labels_line = next(
        line for line in completed.stdout.splitlines() if line.startswith("labels:")
    )
    labels = {label.strip() for label in labels_line.removeprefix("labels:").split(",")}
    assert labels == {"app:cap", "type:report"}
    assert completed.stdout.rstrip().endswith("next: done")
    return [
        "the real shared CLI issue create path adds app identity and type:report",
        "the dry-run path preserves the executable terminal next marker",
    ]
