#!/usr/bin/env python3
"""MVP performance CPython 3.12 identity checker (closes #2572).

Parent: #2530 (performance gate suite).

Probes the comparison Python interpreter (defaults to whatever
`python3` resolves to on PATH) and validates that its identity
matches the required CPython version declared by
`validation/profiles/performance.toml`'s
`runtime_identity.required_cpython` (currently `"3.12"`).

Why this gate exists
--------------------

A perf gate that says "10× CPython" but quietly compared against
CPython 3.11 (slower interpreter, missing 3.12 optimisations) or
3.13 (faster) is measuring against the wrong baseline. The MVP
release promise is "10× CPython **3.12**" — the gate must prove
the comparison runtime really is 3.12 before the speedup numbers
become meaningful.

Acceptance (issue #2572):

    1. Running against Python 3.11 or 3.13 fails the default MVP perf
       gate. Captured major.minor compared verbatim against the
       policy's required value; mismatch exits 1.
    2. JSON summary includes CPython metadata. `executable`,
       `version`, `version_major_minor`, `implementation_name` all
       surface in the JSON payload.
    3. Existing local debug flow can still opt out intentionally.
       `--local-debug-override` flag OR
       `MAMBA_PERF_LOCAL_DEBUG_OVERRIDE=1` env var converts the
       failure into a clearly-marked pass; the JSON payload sets
       `override_active = true` so a release reviewer can tell at a
       glance whether the run cheated.

Operating modes
---------------

`--python PATH` (default: `python3`)
    Probe this interpreter. The script runs
    `<python> -c "..."` to capture executable + version +
    implementation name.

`--identity-json PATH`
    Skip probing and load the identity from a pre-collected JSON
    file. Useful in CI where the bench runner has already captured
    identity and the checker should not re-spawn the interpreter.

`--policy PATH` (default: validation/profiles/performance.toml)
    Read `runtime_identity.required_cpython` from this manifest.

`--local-debug-override`
    Treat a version mismatch as a pass for the purposes of exit code.
    The JSON payload still records the mismatch so a release reviewer
    can tell.

`--format text|json` (default: text)

Exit codes
----------

    0   identity matches OR override active.
    1   version mismatch with no override.
    100 usage / argument error.
    101 unable to probe the interpreter or read identity JSON.

Pure stdlib (Python 3.11+ for tomllib).
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path


SCHEMA_VERSION = 1
EXIT_USAGE = 100
EXIT_IO = 101
EXIT_FAIL = 1

DEFAULT_REQUIRED_CPYTHON = "3.12"
DEFAULT_IMPLEMENTATION = "cpython"
OVERRIDE_ENV_VAR = "MAMBA_PERF_LOCAL_DEBUG_OVERRIDE"

# Probe script — invoked under the target Python. Emits three lines:
#   line 1: sys.executable
#   line 2: sys.version (single physical line — strip newlines)
#   line 3: sys.implementation.name
PROBE_CODE = (
    "import sys;"
    "print(sys.executable);"
    "print(sys.version.replace('\\n', ' '));"
    "print(sys.implementation.name)"
)


@dataclass
class Identity:
    executable: str
    version: str
    version_major_minor: str
    implementation_name: str


def _die(code: int, msg: str) -> None:
    sys.stderr.write(f"cpython_identity_check: {msg}\n")
    sys.exit(code)


def _probe(python_bin: str) -> Identity:
    try:
        proc = subprocess.run(
            [python_bin, "-c", PROBE_CODE],
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        _die(EXIT_IO, f"failed to probe {python_bin!r}: {exc}")
        return Identity("", "", "", "")
    if proc.returncode != 0:
        _die(
            EXIT_IO,
            f"probe of {python_bin!r} exited {proc.returncode}: {proc.stderr.strip()}",
        )
    lines = proc.stdout.splitlines()
    if len(lines) < 3:
        _die(
            EXIT_IO,
            f"probe of {python_bin!r} produced {len(lines)} lines, expected 3: {proc.stdout!r}",
        )
    executable, version, impl = lines[0].strip(), lines[1].strip(), lines[2].strip()
    major_minor = _extract_major_minor(version)
    return Identity(executable, version, major_minor, impl)


def _extract_major_minor(version: str) -> str:
    """Pull "X.Y" from a sys.version string.

    sys.version looks like '3.12.4 (main, ...)'. We take the first
    whitespace-delimited token, then keep only the first two
    dot-segments. Returns '' if the version string is unparseable.
    """
    if not version:
        return ""
    first = version.split()[0]
    parts = first.split(".")
    if len(parts) < 2:
        return ""
    return f"{parts[0]}.{parts[1]}"


def _load_identity_json(path: Path) -> Identity:
    if not path.is_file():
        _die(EXIT_IO, f"identity JSON missing: {path}")
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        _die(EXIT_IO, f"identity JSON invalid ({exc}): {path}")
        return Identity("", "", "", "")
    if not isinstance(data, dict):
        _die(EXIT_IO, f"identity JSON must be an object: {path}")
    executable = str(data.get("executable", ""))
    version = str(data.get("version", ""))
    impl = str(data.get("implementation_name", ""))
    mm = data.get("version_major_minor")
    if isinstance(mm, str) and mm:
        major_minor = mm
    else:
        major_minor = _extract_major_minor(version)
    return Identity(executable, version, major_minor, impl)


def _load_required(policy_path: Path) -> str:
    if not policy_path.is_file():
        return DEFAULT_REQUIRED_CPYTHON
    try:
        data = tomllib.loads(policy_path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as exc:
        _die(EXIT_IO, f"policy invalid TOML ({exc}): {policy_path}")
        return DEFAULT_REQUIRED_CPYTHON
    runtime = data.get("runtime_identity") or {}
    return str(runtime.get("required_cpython", DEFAULT_REQUIRED_CPYTHON))


def _format_text(
    identity: Identity,
    required: str,
    matches: bool,
    override_active: bool,
    exit_code: int,
) -> str:
    lines = [
        f"cpython_identity_check: required={required} actual={identity.version_major_minor or '<unknown>'}",
        f"  executable={identity.executable}",
        f"  version={identity.version}",
        f"  implementation_name={identity.implementation_name}",
    ]
    if not matches and override_active:
        lines.append(
            f"cpython_identity_check: MISMATCH but local-debug override active "
            f"(exit code forced to 0); release runs must NOT use --local-debug-override"
        )
    elif not matches:
        lines.append(
            f"cpython_identity_check: FAIL — runtime is not CPython {required}; "
            f"local debug only: re-run with --local-debug-override or "
            f"{OVERRIDE_ENV_VAR}=1"
        )
    else:
        lines.append("cpython_identity_check: identity matches; clean")
    lines.append(f"exit_code={exit_code}")
    return "\n".join(lines) + "\n"


def _format_json(
    identity: Identity,
    required: str,
    matches: bool,
    override_active: bool,
    exit_code: int,
) -> str:
    payload = {
        "schema_version": SCHEMA_VERSION,
        "executable": identity.executable,
        "version": identity.version,
        "version_major_minor": identity.version_major_minor,
        "implementation_name": identity.implementation_name,
        "required_cpython": required,
        "required_implementation": DEFAULT_IMPLEMENTATION,
        "matches": matches,
        "override_active": override_active,
        "exit_code": exit_code,
    }
    return json.dumps(payload, indent=2, sort_keys=True) + "\n"


def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="cpython_identity_check",
        description="Record CPython 3.12 executable identity in perf gate (#2572).",
    )
    p.add_argument(
        "--python",
        type=str,
        default="python3",
        help="interpreter to probe (default: python3)",
    )
    p.add_argument(
        "--identity-json",
        type=Path,
        default=None,
        help="skip probing; read identity from JSON",
    )
    p.add_argument(
        "--policy",
        type=Path,
        default=None,
        help="path to performance.toml (default: relative to this script)",
    )
    p.add_argument(
        "--local-debug-override",
        action="store_true",
        help="convert mismatch into pass (local debugging only — DO NOT use in release)",
    )
    p.add_argument("--format", choices=("text", "json"), default="text")
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    ns = _parse_args(list(sys.argv[1:] if argv is None else argv))

    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    policy_path = ns.policy or (
        project_root / "validation" / "profiles" / "performance.toml"
    )
    required = _load_required(policy_path.resolve())

    if ns.identity_json is not None:
        identity = _load_identity_json(ns.identity_json.resolve())
    else:
        identity = _probe(ns.python)

    impl_ok = identity.implementation_name == DEFAULT_IMPLEMENTATION
    version_ok = identity.version_major_minor == required
    matches = impl_ok and version_ok

    env_override = os.environ.get(OVERRIDE_ENV_VAR, "") == "1"
    override_active = ns.local_debug_override or env_override

    if matches:
        exit_code = 0
    elif override_active:
        exit_code = 0
    else:
        exit_code = EXIT_FAIL

    if ns.format == "json":
        sys.stdout.write(
            _format_json(identity, required, matches, override_active, exit_code)
        )
    else:
        sys.stderr.write(
            _format_text(identity, required, matches, override_active, exit_code)
        )
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
