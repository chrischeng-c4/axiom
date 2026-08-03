"""Black-box fixture helpers for hand-authored Python EC implementations."""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
from collections.abc import Callable, Iterator
from contextlib import contextmanager
from pathlib import Path
from typing import Any


REPOSITORY_ROOT = Path(__file__).resolve().parents[4]
AW_BINARY = REPOSITORY_ROOT / "target" / "debug" / "aw"
EVIDENCE_ROOT = Path(__file__).resolve().parents[1] / "evidence"

# A Python EC must invoke the controller-provided Rust binary, never turn into
# an implicit Cargo/rustup build client.  Checking the ordinary executable bit
# alone would allow a shell script or unrelated command to be launched as AW.
# The supported AW distribution targets are native Mach-O, ELF, or PE files.
_NATIVE_EXECUTABLE_MAGICS = frozenset(
    {
        b"\x7fELF",
        b"\xfe\xed\xfa\xce",
        b"\xce\xfa\xed\xfe",
        b"\xfe\xed\xfa\xcf",
        b"\xcf\xfa\xed\xfe",
        b"\xca\xfe\xba\xbe",
        b"\xbe\xba\xfe\xca",
    }
)


def _is_native_aw_executable(candidate: Path) -> bool:
    """Return whether `candidate` looks like a supported native AW binary."""
    try:
        with candidate.open("rb") as stream:
            header = stream.read(4096)
    except OSError:
        return False
    if header[:4] == b"\x7fELF":
        return _is_valid_elf_header(header)
    if header[:4] in _NATIVE_EXECUTABLE_MAGICS:
        return _is_valid_macho_header(header)
    if header[:2] == b"MZ":
        return _is_valid_pe_header(header)
    return False


def _is_valid_elf_header(header: bytes) -> bool:
    """Reject truncated/corrupt ELF files before they reach `execve`."""
    if len(header) < 64 or header[4] not in (1, 2) or header[5] not in (1, 2):
        return False
    if header[6] != 1:
        return False
    endian = "little" if header[5] == 1 else "big"
    if int.from_bytes(header[20:24], endian) != 1:
        return False
    elf_class = header[4]
    header_size_offset = 40 if elf_class == 1 else 52
    expected_header_size = 52 if elf_class == 1 else 64
    return int.from_bytes(
        header[header_size_offset : header_size_offset + 2], endian
    ) == expected_header_size


def _is_valid_macho_header(header: bytes) -> bool:
    """Reject truncated/corrupt Mach-O files before they reach `execve`."""
    magic = header[:4]
    if magic in (b"\xfe\xed\xfa\xce", b"\xce\xfa\xed\xfe"):
        minimum_size = 28
    elif magic in (b"\xfe\xed\xfa\xcf", b"\xcf\xfa\xed\xfe"):
        minimum_size = 32
    else:
        # A universal/fat header contains a non-zero architecture count plus
        # at least one complete architecture descriptor.
        if len(header) < 8:
            return False
        endian = "big" if magic == b"\xca\xfe\xba\xbe" else "little"
        architectures = int.from_bytes(header[4:8], endian)
        return architectures > 0 and len(header) >= 8 + (architectures * 20)
    if len(header) < minimum_size:
        return False
    endian = "big" if magic[:1] == b"\xfe" else "little"
    commands = int.from_bytes(header[16:20], endian)
    commands_size = int.from_bytes(header[20:24], endian)
    return commands > 0 and commands_size > 0 and len(header) >= minimum_size + commands_size


def _is_valid_pe_header(header: bytes) -> bool:
    """Reject a DOS stub without a PE signature before it is executed."""
    if len(header) < 64:
        return False
    pe_offset = int.from_bytes(header[0x3C:0x40], "little")
    return pe_offset >= 64 and pe_offset + 4 <= len(header) and header[pe_offset : pe_offset + 4] == b"PE\0\0"


def resolve_aw_binary() -> Path:
    """Resolve and preflight a controller-supplied AW executable without building."""
    override = os.environ.get("AW_EC_AW_BINARY")
    candidate = (
        Path(override).resolve()
        if override
        else (REPOSITORY_ROOT / "target" / "debug" / "aw").resolve()
    )
    if not candidate.is_file():
        raise AssertionError(
            f"Prebuilt AW binary missing or not a regular file at {candidate}. "
            "The controller or environment must build or supply the executable separately."
        )
    if not os.access(candidate, os.X_OK):
        raise AssertionError(
            f"Prebuilt AW binary at {candidate} is not executable. "
            "The controller or environment must build or supply the executable separately."
        )
    if not _is_native_aw_executable(candidate):
        raise AssertionError(
            f"Prebuilt AW binary at {candidate} is not a supported native executable. "
            "The controller or environment must build or supply the executable separately."
        )
    _preflight_aw_binary(candidate)
    return candidate


def _preflight_aw_binary(candidate: Path) -> None:
    """Prove the configured binary launches as AW before running a product command."""
    try:
        completed = subprocess.run(
            [str(candidate), "--version"],
            text=True,
            capture_output=True,
            check=False,
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise AssertionError(
            f"Prebuilt AW binary at {candidate} could not execute its AW preflight: {error}. "
            "The controller or environment must build or supply the executable separately."
        ) from error
    version = completed.stdout.strip()
    if completed.returncode != 0 or not version.startswith("aw "):
        raise AssertionError(
            f"Prebuilt AW binary at {candidate} failed AW preflight "
            f"(exit={completed.returncode}, stdout={version!r}). "
            "The controller or environment must build or supply the executable separately."
        )


def _ensure_aw_binary() -> Path:
    """Compatibility preflight for legacy ECs; it never builds the binary.

    Older cases import this helper before invoking the historical `AW_BINARY`
    constant directly.  Keeping the symbol avoids breaking those production
    cases while preserving the new controller-owned prebuilt-binary boundary.
    """
    return resolve_aw_binary()


@contextmanager
def project_fixture() -> Iterator[Path]:
    with tempfile.TemporaryDirectory(prefix="aw-python-ec-") as raw_root:
        root = Path(raw_root)
        (root / "aw.toml").write_text(
            """
[agentic_workflow.workspace]
mode = "in_place"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "demo"
label = "app:demo"
path = "."
tech_design_path = "tech-design"

[[projects.workspaces]]
name = "demo"
paths = ["**"]
target = "rust"
""".lstrip(),
            encoding="utf-8",
        )
        yield root


def write_python_artifact_unit_test(artifact_root: Path, name: str = "fixture") -> Path:
    """Write the `tests/unit/test_*.py` a canonical Python artifact requires.

    `aw ec check` refuses to read an artifact with no authored unit tests, so
    the fixture needs one before any contract assertion becomes reachable.
    """
    unit_dir = artifact_root / "tests/unit"
    unit_dir.mkdir(parents=True, exist_ok=True)
    test_path = unit_dir / f"test_{name}.py"
    class_name = "".join(part.capitalize() for part in name.split("_"))
    test_path.write_text(
        "import unittest\n"
        "\n"
        "\n"
        f"class {class_name}ManifestTest(unittest.TestCase):\n"
        f"    def test_{name}_declares_a_python_project_manifest(self) -> None:\n"
        "        from pathlib import Path\n"
        "\n"
        "        root = Path(__file__).resolve().parents[2]\n"
        '        manifest = (root / "pyproject.toml").read_text(encoding="utf-8")\n'
        '        self.assertIn("[project]", manifest)\n'
        '        self.assertIn("requires-python", manifest)\n'
        "\n"
        "\n"
        'if __name__ == "__main__":\n'
        "    unittest.main()\n",
        encoding="utf-8",
    )
    return test_path


def git_commit_fixture(root: Path, message: str = "fixture") -> None:
    """Make `root` a committed git repository.

    `aw` resolves native target ownership through `git log`, so a fixture that
    reaches ownership resolution without a repository fails on the environment
    rather than on the contract under test.
    """
    for args in (
        ("init",),
        ("config", "user.email", "fixture@example.com"),
        ("config", "user.name", "Fixture"),
        ("add", "-A"),
        ("commit", "-m", message),
    ):
        completed = subprocess.run(
            ["git", *args],
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
        )
        if completed.returncode != 0:
            raise AssertionError(
                f"git {' '.join(args)} failed in fixture: {completed.stderr}"
            )


def write_python_artifact_lock(
    artifact_root: Path,
    *,
    name: str,
    version: str = "0.1.0",
    requires_python: str = ">=3.11",
) -> Path:
    """Write the `uv.lock` a fixture manifest declares in `dependency_files`.

    A Python artifact whose `dependency_files` names `uv.lock` cannot be read at
    all when the lock is absent, so every assertion behind it is unreachable.
    This mirrors the dependency-free lock `refresh_uv_lock` synthesizes in
    `apps/agentic-workflow/src/services/python_artifact.rs` so the fixture is
    frozen-executable rather than merely present.
    """
    lock = artifact_root / "uv.lock"
    lock.write_text(
        "version = 1\n"
        "revision = 3\n"
        f'requires-python = "{requires_python}"\n'
        "\n"
        "[[package]]\n"
        f'name = "{name}"\n'
        f'version = "{version}"\n'
        'source = { virtual = "." }\n',
        encoding="utf-8",
    )
    return lock


def run_aw(
    root: Path,
    *args: str,
    expect_success: bool | None = True,
    env_overrides: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    aw_binary = resolve_aw_binary()
    env = os.environ.copy()
    env["AW_FIXTURE_LOCAL_BACKEND"] = "1"
    env["AW_DISABLE_CAP"] = "1"
    if env_overrides:
        env.update(env_overrides)
    completed = subprocess.run(
        [str(aw_binary), *args],
        cwd=root,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if expect_success is True and completed.returncode != 0:
        raise AssertionError(
            f"aw {' '.join(args)} failed:\n"
            f"stdout={completed.stdout}\nstderr={completed.stderr}"
        )
    if expect_success is False and completed.returncode == 0:
        raise AssertionError(f"aw {' '.join(args)} unexpectedly succeeded")
    return completed


def final_json(completed: subprocess.CompletedProcess[str]) -> dict[str, Any]:
    raw = completed.stdout
    decoder = json.JSONDecoder()
    values: list[Any] = []
    cursor = 0
    while cursor < len(raw):
        while cursor < len(raw) and raw[cursor].isspace():
            cursor += 1
        if cursor >= len(raw):
            break
        value, cursor = decoder.raw_decode(raw, cursor)
        values.append(value)
    if not values or not isinstance(values[-1], dict):
        raise AssertionError(f"command emitted no JSON:\nstderr={completed.stderr}")
    return values[-1]


def create(root: Path, title: str, work_item_type: str, *extra: str) -> dict[str, Any]:
    return final_json(
        run_aw(
            root,
            "wi",
            "create",
            "--title",
            title,
            "--type",
            work_item_type,
            "--project",
            "demo",
            *extra,
        )
    )


def show(root: Path, slug: str) -> dict[str, Any]:
    payload = final_json(run_aw(root, "wi", "show", slug))
    issue = payload.get("issue")
    return issue if isinstance(issue, dict) else payload


def record_evidence(case_id: str, assertions: list[str]) -> None:
    EVIDENCE_ROOT.mkdir(parents=True, exist_ok=True)
    (EVIDENCE_ROOT / f"{case_id}.json").write_text(
        json.dumps(
            {
                "protocol": "aw.python-ec.evidence.v1",
                "case_id": case_id,
                "status": "passed",
                "assertions": assertions,
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )


def verify_case(case_id: str, verifier: Callable[[], list[str]]) -> None:
    record_evidence(case_id, verifier())
