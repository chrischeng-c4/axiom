"""External contract for stage-specific static diagnostic normalization."""

from guard_contract import (
    assert_finding,
    assert_scan_consistency,
    expected_finding_id_prefix,
    expected_finding_source,
    fixture,
    run_guard,
)

DIMENSION = "security"


def verify() -> list[str]:
    with fixture(
        {
            "unsafe.js": "eval('alert(1)');\n",
            "unsafe.py": (
                "import subprocess\n"
                "subprocess.run(\n"
                "    'id',\n"
                "    shell=True,\n"
                ")\n"
            ),
            "Dockerfile": "FROM ubuntu:latest\n",
        }
    ) as root:
        _, report = run_guard(
            [
                "scan",
                str(root),
                "--profile",
                "security-lint",
                "--compact",
                "--no-persist",
            ],
            expected_exit_codes={1},
        )
    assertions = assert_scan_consistency(report)
    expected = {
        "JS004": (root / "unsafe.js", 1, "high"),
        "PY304": (root / "unsafe.py", 2, "medium"),
        "DK002": (root / "Dockerfile", 1, "medium"),
    }
    if report["summary"]["security_findings"] != len(expected):
        raise AssertionError(
            "diagnostic fixture must emit exactly JS004, PY304, and DK002"
        )
    source = expected_finding_source()
    for rule, (expected_path, line, severity) in expected.items():
        finding = assert_finding(report, rule=rule)
        location = finding.get("location", {})
        evidence = finding.get("evidence", {})
        if evidence.get("source") != source:
            raise AssertionError(f"{rule} diagnostic source is not {source}")
        if (
            location.get("start_line") != line
            or location.get("path") != str(expected_path)
        ):
            raise AssertionError(
                f"{rule} diagnostic location was not preserved: {location!r}"
            )
        expected_id = _expected_finding_id(rule, expected_path, line)
        if finding.get("id") != expected_id:
            raise AssertionError(
                f"{rule} finding id drifted: {finding.get('id')!r} != {expected_id!r}"
            )
        if finding.get("severity") != severity:
            raise AssertionError(
                f"{rule} severity drifted: {finding.get('severity')!r}"
            )
    assertions.extend(
        [
            "exactly eval, multiline shell=True, and latest-tag diagnostics become Guard findings",
            f"all three finding identities honestly preserve {source} and their rule",
            "all three findings preserve exact file, line, and severity projections",
        ]
    )
    return assertions


def _expected_finding_id(rule: str, path: object, line: int) -> str:
    subject = f"{path}:{line}"
    squashed = "".join(
        character if character.isalnum() or character in "-_" else "-"
        for character in subject
    )
    return f"{expected_finding_id_prefix(rule)}{squashed}"
