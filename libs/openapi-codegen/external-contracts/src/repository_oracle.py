"""Repository-facing oracle implementation; independent from product tests."""

from __future__ import annotations

import json
import re
import subprocess
import tomllib
from pathlib import Path
from typing import Callable

from active_reference_boundary import (
    ACTIVE_WORKTREE_INVENTORY_COMMAND,
    GENERATED_EVIDENCE_PREFIX,
    HISTORICAL_ALLOWLIST,
)
from consumer_boundary import CONSUMERS, DEPENDENCY_NAME, DEPENDENCY_VERSION
from evidence_schema import failed, passed
from git_version_boundary import CONTRACT, LEGACY_IDENTITIES
from target_matrix_boundary import MATRIX_CASES


def _guard(check_id: str, action: Callable[[], tuple[int, dict]]) -> dict:
    try:
        count, details = action()
        if count <= 0:
            raise AssertionError("oracle produced no observations")
        return passed(check_id, count, details)
    except Exception as error:
        return failed(check_id, str(error))


def _load_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def _run(root: Path, command: tuple[str, ...]) -> dict:
    result = subprocess.run(
        command,
        cwd=root,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        raise AssertionError(
            f"{' '.join(command)} exited {result.returncode}: "
            f"{result.stderr[-1200:].strip() or result.stdout[-1200:].strip()}"
        )
    return {
        "command": list(command),
        "exit_code": result.returncode,
        "stdout_tail": result.stdout[-1200:],
    }


def check_identity(root: Path) -> list[dict]:
    manifest_path = root / "libs/openapi-codegen/Cargo.toml"
    source_path = root / "libs/openapi-codegen/src/lib.rs"

    def metadata() -> tuple[int, dict]:
        result = subprocess.run(
            ("cargo", "metadata", "--format-version", "1", "--no-deps"),
            cwd=root,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        assert result.returncode == 0, result.stderr[-1200:]
        document = json.loads(result.stdout)
        expected_manifest = str(manifest_path.resolve())
        matches = [
            package
            for package in document["packages"]
            if package.get("manifest_path") == expected_manifest
        ]
        assert len(matches) == 1, f"expected one metadata package, got {len(matches)}"
        package = matches[0]
        assert package["name"] == CONTRACT.package, package["name"]
        assert package["version"] == CONTRACT.version, package["version"]
        return 1, {"name": package["name"], "version": package["version"]}

    def manifest() -> tuple[int, dict]:
        package = _load_toml(manifest_path)["package"]
        observed = {
            "name": package.get("name"),
            "version": package.get("version"),
            "publish": package.get("publish"),
        }
        expected = {
            "name": CONTRACT.package,
            "version": CONTRACT.version,
            "publish": CONTRACT.publish,
        }
        assert observed == expected, f"expected {expected!r}, got {observed!r}"
        return len(observed), observed

    def crate() -> tuple[int, dict]:
        observed = _load_toml(manifest_path).get("lib", {}).get("name")
        assert observed == CONTRACT.crate, f"expected {CONTRACT.crate}, got {observed}"
        return 1, {"crate": observed}

    source = source_path.read_text()

    def source_identity(check_id: str, declaration: str, legacy: str) -> tuple[int, dict]:
        assert declaration in source, f"exact declaration {declaration!r} is absent"
        assert legacy not in source, f"superseded identity {legacy!r} remains"
        return 1, {check_id: declaration}

    def release_tag() -> tuple[int, dict]:
        expected = f"{CONTRACT.package}@{CONTRACT.version}"
        assert CONTRACT.tag == expected, f"tag must be {expected}"
        return 1, {"tag": CONTRACT.tag}

    return [
        _guard("cargo-metadata-package", metadata),
        _guard("manifest-package-version-publish", manifest),
        _guard("rust-crate-name", crate),
        _guard(
            "sidecar-filename",
            lambda: source_identity(
                "sidecar",
                f'pub const MANIFEST_FILE: &str = "{CONTRACT.manifest}";',
                LEGACY_IDENTITIES[2],
            ),
        ),
        _guard(
            "generator-identity",
            lambda: source_identity(
                "generator",
                f'generator: "{CONTRACT.package}".to_string()',
                LEGACY_IDENTITIES[0],
            ),
        ),
        _guard("release-tag", release_tag),
    ]


def check_active_references(root: Path) -> list[dict]:
    def sweep() -> tuple[int, dict]:
        inventory = subprocess.run(
            ACTIVE_WORKTREE_INVENTORY_COMMAND,
            cwd=root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        assert inventory.returncode == 0, inventory.stderr.decode(errors="replace")[-1200:]
        text_files: list[tuple[Path, str]] = []
        binary_files_skipped = 0
        for raw_path in inventory.stdout.split(b"\0"):
            if not raw_path:
                continue
            relative_path = raw_path.decode(errors="surrogateescape")
            candidate = root / relative_path
            if not candidate.is_file():
                continue
            normalized = candidate.relative_to(root).as_posix()
            if normalized in HISTORICAL_ALLOWLIST:
                continue
            if normalized.startswith(GENERATED_EVIDENCE_PREFIX):
                continue
            if normalized == ".aw" or normalized.startswith(".aw/"):
                continue
            if any(part in {".git", "__pycache__", "target"} for part in candidate.parts):
                continue
            content = candidate.read_bytes()
            if b"\0" in content:
                binary_files_skipped += 1
                continue
            text_files.append((candidate, content.decode(errors="replace")))
        findings: list[dict[str, object]] = []
        for candidate, text in sorted(text_files, key=lambda item: item[0].as_posix()):
            for identity in LEGACY_IDENTITIES:
                lines = [
                    index
                    for index, line in enumerate(text.splitlines(), start=1)
                    if identity in line
                ]
                if lines:
                    findings.append(
                        {
                            "path": candidate.relative_to(root).as_posix(),
                            "identity": identity,
                            "lines": lines,
                        }
                    )
        assert not findings, f"unallowlisted superseded identities remain: {findings!r}"
        return len(text_files), {
            "scanned_text_files": len(text_files),
            "binary_files_skipped": binary_files_skipped,
            "text_detection": "no NUL byte",
            "historical_allowlist": sorted(HISTORICAL_ALLOWLIST),
            "generated_evidence_prefix": GENERATED_EVIDENCE_PREFIX,
            "forbidden_identity_count": len(LEGACY_IDENTITIES),
            "inventory_command": list(ACTIVE_WORKTREE_INVENTORY_COMMAND),
        }

    return [_guard("active-reference-sweep", sweep)]


def check_target_matrix(root: Path) -> list[dict]:
    def execute_exact_test(matrix_case) -> tuple[int, dict]:
        output = _run(root, matrix_case.command)
        stdout = output["stdout_tail"]
        marker = f"test {matrix_case.test_name} ... ok"
        assert marker in stdout, (
            f"Cargo exited successfully but did not report the exact test marker {marker!r}"
        )
        summaries = re.findall(
            r"test result: ok\. (\d+) passed; (\d+) failed; "
            r"(\d+) ignored; (\d+) measured; (\d+) filtered out",
            stdout,
        )
        assert summaries, "Cargo emitted no parseable test-result summary"
        passed_count, failed_count, ignored_count, measured_count, filtered_count = (
            map(int, summaries[-1])
        )
        assert (passed_count, failed_count, ignored_count, measured_count) == (1, 0, 0, 0), (
            f"expected exactly one executed passing test, got {summaries[-1]!r}"
        )
        output["test_name"] = matrix_case.test_name
        output["passed_count"] = passed_count
        output["filtered_count"] = filtered_count
        return passed_count, output

    results: list[dict] = []
    for matrix_case in MATRIX_CASES:
        results.append(
            _guard(
                matrix_case.check_id,
                lambda matrix_case=matrix_case: execute_exact_test(matrix_case),
            )
        )
    return results


def check_consumers(root: Path) -> list[dict]:
    results: list[dict] = []
    for consumer in CONSUMERS:
        def verify(consumer=consumer) -> tuple[int, dict]:
            manifest = _load_toml(root / consumer.manifest)
            dependencies = manifest.get("dependencies", {})
            entry = dependencies.get(DEPENDENCY_NAME)
            assert isinstance(entry, dict), (
                f"{consumer.manifest} must declare {DEPENDENCY_NAME} as an inline table"
            )
            observed = {
                "path": entry.get("path"),
                "version": entry.get("version"),
                "package": entry.get("package"),
            }
            assert observed["path"] == consumer.dependency_path, observed
            assert observed["version"] == DEPENDENCY_VERSION, observed
            assert observed["package"] is None, "dependency aliases are not allowed"
            command = _run(root, consumer.command)
            return 2, {
                "manifest": consumer.manifest,
                "dependency": {
                    "name": DEPENDENCY_NAME,
                    "path": observed["path"],
                    "version": observed["version"],
                },
                "verification": command,
            }

        results.append(_guard(consumer.check_id, verify))
    return results
