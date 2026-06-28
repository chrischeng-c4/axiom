#!/usr/bin/env python3
"""Materialize the pinned CPython oracle environment.

The Rust harness and sweep.py prefer:

    projects/mamba/tests/cpython/.cache/oracle-env/bin/python3

when MAMBA_ORACLE_PYTHON is not set. This tool creates that venv from the
checked-in oracle-env requirements lock, copies CPython's Lib/test package into
site-packages, and writes a content stamp so unchanged re-runs are cheap.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
import sysconfig
from hashlib import sha256
from pathlib import Path
from typing import Any


SCRIPT_SCHEMA = 1
SCRIPT_VERSION = "ensure_oracle_env.py:v1"

TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parents[3]
FIXTURES_DIR = MAMBA_DIR / "tests" / "cpython"
DEFAULT_ENV_DIR = FIXTURES_DIR / ".cache" / "oracle-env"
DEFAULT_REQUIREMENTS = TOOLS_DIR.parent / "config" / "oracle-env" / "requirements.txt"
STAMP_NAME = ".mamba-oracle-env.json"


def repo_rel(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(MAMBA_DIR.parents[1].resolve()))
    except ValueError:
        return str(path)


def sha256_file(path: Path) -> str:
    h = sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def run_capture(args: list[str], *, cwd: Path | None = None) -> str:
    result = subprocess.run(
        args,
        cwd=str(cwd) if cwd else None,
        text=True,
        capture_output=True,
    )
    if result.returncode != 0:
        stderr = result.stderr.strip()
        stdout = result.stdout.strip()
        detail = stderr or stdout or f"exit {result.returncode}"
        raise RuntimeError(f"{' '.join(args)} failed: {detail}")
    return result.stdout


def run_checked(args: list[str]) -> None:
    print("+", " ".join(args), flush=True)
    subprocess.run(args, check=True)


def python_info(python: str) -> dict[str, Any]:
    script = """
import json
import ntpath
import platform
import sys

print(json.dumps({
    "executable": sys.executable,
    "implementation": platform.python_implementation(),
    "ntpath_allow_missing": hasattr(ntpath, "ALLOW_MISSING"),
    "version": sys.version,
    "version_info": list(sys.version_info[:3]),
}))
"""
    return json.loads(run_capture([python, "-c", script]))


def ensure_oracle_python(info: dict[str, Any], python: str) -> None:
    if info["implementation"] != "CPython" or info["version_info"][:2] != [3, 12]:
        got = f"{info['implementation']} {'.'.join(map(str, info['version_info']))}"
        raise SystemExit(
            f"{python!r} resolves to {got}; oracle-env requires CPython 3.12. "
            "Pass --python /path/to/python3.12."
        )
    if not info["ntpath_allow_missing"]:
        raise SystemExit(
            f"{python!r} is CPython 3.12 but lacks ntpath.ALLOW_MISSING; "
            "use a newer CPython 3.12.x build for the oracle environment."
        )


def resolve_python(arg: str | None) -> str:
    candidates = [arg] if arg else ["python3.12", "python3"]
    errors: list[str] = []
    for candidate in candidates:
        if not candidate:
            continue
        try:
            info = python_info(candidate)
            ensure_oracle_python(info, candidate)
            return str(Path(info["executable"]).resolve())
        except Exception as exc:  # noqa: BLE001
            errors.append(f"{candidate}: {exc}")
    raise SystemExit("No usable CPython 3.12 found:\n  " + "\n  ".join(errors))


def find_lib_test_source(python: str, override: str | None) -> Path:
    if override:
        source = Path(override).expanduser().resolve()
    else:
        script = """
import json
import pathlib
import test

print(json.dumps(str(pathlib.Path(test.__file__).resolve().parent)))
"""
        try:
            source = Path(json.loads(run_capture([python, "-c", script]))).resolve()
        except Exception as exc:  # noqa: BLE001
            raise SystemExit(
                "Could not locate CPython Lib/test from the selected interpreter. "
                "Pass --lib-test-source /path/to/cpython-3.12/Lib/test. "
                f"Original error: {exc}"
            ) from exc
    if not source.is_dir():
        raise SystemExit(f"Lib/test source is not a directory: {source}")
    if not (source / "__init__.py").is_file() or not (source / "support").is_dir():
        raise SystemExit(f"Lib/test source does not look like CPython Lib/test: {source}")
    return source


def env_python(env_dir: Path) -> Path:
    if sys.platform == "win32":
        return env_dir / "Scripts" / "python.exe"
    return env_dir / "bin" / "python3"


def env_site_packages(python: Path) -> Path:
    script = "import json, sysconfig; print(json.dumps(sysconfig.get_paths()['purelib']))"
    return Path(json.loads(run_capture([str(python), "-c", script]))).resolve()


def copy_lib_test(source: Path, env_py: Path) -> Path:
    site_packages = env_site_packages(env_py)
    dest = site_packages / "test"
    if dest.exists():
        shutil.rmtree(dest)

    def ignore(_dir: str, names: list[str]) -> set[str]:
        return {name for name in names if name == "__pycache__" or name.endswith((".pyc", ".pyo"))}

    shutil.copytree(source, dest, ignore=ignore)
    return dest


def identity_inputs(requirements: Path, python: str, python_data: dict[str, Any], lib_test_source: Path) -> dict[str, Any]:
    return {
        "schema": SCRIPT_SCHEMA,
        "script_version": SCRIPT_VERSION,
        "requirements": repo_rel(requirements),
        "requirements_sha256": sha256_file(requirements),
        "python_executable": python,
        "python_implementation": python_data["implementation"],
        "python_ntpath_allow_missing": python_data["ntpath_allow_missing"],
        "python_version": python_data["version"],
        "python_version_info": python_data["version_info"],
        "lib_test_source": str(lib_test_source),
    }


def identity_for(inputs: dict[str, Any]) -> str:
    payload = json.dumps(inputs, sort_keys=True, separators=(",", ":")).encode()
    return sha256(payload).hexdigest()


def read_stamp(env_dir: Path) -> dict[str, Any] | None:
    stamp = env_dir / STAMP_NAME
    if not stamp.is_file():
        return None
    try:
        return json.loads(stamp.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None


def write_stamp(env_dir: Path, identity: str, inputs: dict[str, Any], lib_test_dest: Path) -> None:
    stamp = {
        "identity": identity,
        "inputs": inputs,
        "env_python": str(env_python(env_dir)),
        "lib_test_dest": str(lib_test_dest),
    }
    (env_dir / STAMP_NAME).write_text(json.dumps(stamp, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def up_to_date(env_dir: Path, identity: str) -> tuple[bool, str]:
    py = env_python(env_dir)
    if not py.is_file():
        return False, f"missing {repo_rel(py)}"
    stamp = read_stamp(env_dir)
    if not stamp:
        return False, f"missing or invalid {repo_rel(env_dir / STAMP_NAME)}"
    if stamp.get("identity") != identity:
        return False, "input hash changed"
    lib_test_dest = Path(stamp.get("lib_test_dest", ""))
    if not (lib_test_dest / "support").is_dir():
        return False, "copied Lib/test package is missing"
    return True, "up to date"


def sync_env(env_dir: Path, requirements: Path, python: str, uv: str, lib_test_source: Path) -> Path:
    py = env_python(env_dir)
    if not py.is_file():
        run_checked([uv, "venv", str(env_dir), "--python", python])
        py = env_python(env_dir)
    run_checked([uv, "pip", "sync", "--python", str(py), str(requirements)])
    return copy_lib_test(lib_test_source, py)


def atomic_rebuild(env_dir: Path, requirements: Path, python: str, uv: str, lib_test_source: Path) -> Path:
    parent = env_dir.parent
    parent.mkdir(parents=True, exist_ok=True)
    tmp = parent / f"{env_dir.name}.tmp.{os.getpid()}"
    old = parent / f"{env_dir.name}.old.{os.getpid()}"
    if tmp.exists():
        shutil.rmtree(tmp)
    if old.exists():
        shutil.rmtree(old)

    try:
        lib_test_dest = sync_env(tmp, requirements, python, uv, lib_test_source)
        if env_dir.exists():
            env_dir.rename(old)
        tmp.rename(env_dir)
        if old.exists():
            shutil.rmtree(old)
        return env_dir / lib_test_dest.relative_to(tmp)
    except Exception:
        if tmp.exists():
            shutil.rmtree(tmp)
        if old.exists() and not env_dir.exists():
            old.rename(env_dir)
        raise


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Create or refresh projects/mamba/tests/cpython/.cache/oracle-env "
            "from the pinned CPython oracle requirements lock."
        )
    )
    parser.add_argument("--env-dir", default=str(DEFAULT_ENV_DIR), help="oracle venv path")
    parser.add_argument("--requirements", default=str(DEFAULT_REQUIREMENTS), help="requirements lock path")
    parser.add_argument("--python", help="CPython 3.12 interpreter used to create the venv")
    parser.add_argument("--lib-test-source", help="CPython 3.12 Lib/test directory to copy into site-packages")
    parser.add_argument("--uv", default="uv", help="uv executable")
    parser.add_argument("--force", action="store_true", help="rebuild even when the stamp is current")
    parser.add_argument("--check", action="store_true", help="only verify that the env is current")
    parser.add_argument("--dry-run", action="store_true", help="print the planned action without changing files")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    env_dir = Path(args.env_dir).expanduser().resolve()
    requirements = Path(args.requirements).expanduser().resolve()
    if not requirements.is_file():
        raise SystemExit(f"requirements lock is missing: {requirements}")

    python = resolve_python(args.python)
    python_data = python_info(python)
    lib_test_source = find_lib_test_source(python, args.lib_test_source)
    inputs = identity_inputs(requirements, python, python_data, lib_test_source)
    identity = identity_for(inputs)
    current, reason = up_to_date(env_dir, identity)

    if args.check:
        if current:
            print(f"oracle-env up to date: {repo_rel(env_python(env_dir))}", flush=True)
            return 0
        print(f"oracle-env not current: {reason}", file=sys.stderr)
        return 1

    if current and not args.force:
        print(f"oracle-env up to date: {repo_rel(env_python(env_dir))}", flush=True)
        return 0

    action = "rebuild" if args.force or not env_python(env_dir).is_file() else "refresh"
    print(f"oracle-env {action}: {reason}", flush=True)
    print(f"env: {repo_rel(env_dir)}", flush=True)
    print(f"requirements: {repo_rel(requirements)}", flush=True)
    print(f"python: {python_data['version'].split()[0]} at {python}", flush=True)
    print(f"Lib/test: {lib_test_source}", flush=True)
    if args.dry_run:
        return 0

    if args.force or not env_python(env_dir).is_file():
        lib_test_dest = atomic_rebuild(env_dir, requirements, python, args.uv, lib_test_source)
    else:
        lib_test_dest = sync_env(env_dir, requirements, python, args.uv, lib_test_source)
    write_stamp(env_dir, identity, inputs, lib_test_dest)
    print(f"oracle-env ready: {repo_rel(env_python(env_dir))}", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
