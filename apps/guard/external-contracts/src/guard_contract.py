"""Source-independent oracle helpers for Guard's public CLI contract."""

from __future__ import annotations

import json
import hashlib
import os
import subprocess
import tempfile
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import Iterator


REPOSITORY_ROOT = Path(__file__).resolve().parents[4]
TECH_DESIGN_ROOT = REPOSITORY_ROOT / "apps/guard/tech-design"
TECH_DESIGN_ENTRYPOINT = TECH_DESIGN_ROOT / "src/cli.py"
ADAPTER_STUB = Path(__file__).resolve().with_name("adapter_stub.py")
DYNAMIC_TMP_ROOT = REPOSITORY_ROOT / "target/guard-python-ec"
WORKSPACE_GUARD = REPOSITORY_ROOT / "target/debug/guard"
WORKSPACE_GUARD_RECEIPT = DYNAMIC_TMP_ROOT / "workspace-guard-build.json"
STANDALONE_TARGET = DYNAMIC_TMP_ROOT / "standalone-build"
STANDALONE_GUARD = STANDALONE_TARGET / "debug/guard"
_BUILT_WORKSPACE_GUARD: Path | None = None


def _file_digest(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def _workspace_rust_digest() -> str:
    inputs: list[Path] = []
    ignored_directories = {
        ".git",
        "target",
        "node_modules",
        ".venv",
        "venv",
        "__pycache__",
    }
    for directory, names, files in os.walk(REPOSITORY_ROOT):
        names[:] = sorted(name for name in names if name not in ignored_directories)
        root = Path(directory)
        for name in sorted(files):
            path = root / name
            relative = path.relative_to(REPOSITORY_ROOT)
            if (
                name.endswith(".rs")
                or name in {"Cargo.toml", "Cargo.lock", "build.rs"}
                or name.startswith("rust-toolchain")
                or relative.as_posix().startswith(".cargo/config")
            ):
                inputs.append(path)
    hasher = hashlib.sha256()
    for path in sorted(inputs):
        relative = path.relative_to(REPOSITORY_ROOT).as_posix().encode()
        content = path.read_bytes()
        hasher.update(relative)
        hasher.update(b"\0")
        hasher.update(len(content).to_bytes(8, "big"))
        hasher.update(b"\0")
        hasher.update(content)
    return "sha256:" + hasher.hexdigest()


def guard_binary() -> Path:
    global _BUILT_WORKSPACE_GUARD
    configured = os.environ.get("GUARD_BIN")
    if configured:
        candidate = Path(configured).resolve()
        expected_digest = os.environ.get("GUARD_BIN_SHA256")
        if not candidate.is_file() or not expected_digest:
            raise AssertionError(
                "GUARD_BIN requires an existing file and digest-bound GUARD_BIN_SHA256"
            )
        actual_digest = _file_digest(candidate)
        if actual_digest != expected_digest:
            raise AssertionError(
                f"GUARD_BIN digest mismatch: {actual_digest} != {expected_digest}"
            )
        return candidate
    if _BUILT_WORKSPACE_GUARD is not None:
        return _BUILT_WORKSPACE_GUARD
    source_digest = _workspace_rust_digest()
    if WORKSPACE_GUARD.is_file() and WORKSPACE_GUARD_RECEIPT.is_file():
        try:
            receipt = json.loads(WORKSPACE_GUARD_RECEIPT.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError):
            receipt = {}
        if (
            isinstance(receipt, dict)
            and receipt.get("source_digest") == source_digest
            and receipt.get("binary_digest") == _file_digest(WORKSPACE_GUARD)
        ):
            _BUILT_WORKSPACE_GUARD = WORKSPACE_GUARD.resolve()
            return _BUILT_WORKSPACE_GUARD
    completed = subprocess.run(
        ["cargo", "build", "-p", "guard", "--bin", "guard"],
        cwd=REPOSITORY_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0 or not WORKSPACE_GUARD.is_file():
        raise AssertionError(
            "current-worktree Guard build failed; "
            f"exit={completed.returncode}; stderr={completed.stderr!r}"
        )
    _BUILT_WORKSPACE_GUARD = WORKSPACE_GUARD.resolve()
    DYNAMIC_TMP_ROOT.mkdir(parents=True, exist_ok=True)
    WORKSPACE_GUARD_RECEIPT.write_text(
        json.dumps(
            {
                "protocol": "guard-python-ec.workspace-build.v1",
                "source_digest": source_digest,
                "binary_digest": _file_digest(_BUILT_WORKSPACE_GUARD),
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    return _BUILT_WORKSPACE_GUARD


def build_standalone_guard() -> Path:
    """Build the exact public package/binary pair without an override or cache receipt."""
    environment = os.environ.copy()
    environment.pop("GUARD_BIN", None)
    environment.pop("GUARD_BIN_SHA256", None)
    environment["CARGO_TARGET_DIR"] = str(STANDALONE_TARGET)
    completed = subprocess.run(
        ["cargo", "build", "-p", "guard", "--bin", "guard"],
        cwd=REPOSITORY_ROOT,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0 or not STANDALONE_GUARD.is_file():
        raise AssertionError(
            "standalone Guard package/binary build failed; "
            f"exit={completed.returncode}; stderr={completed.stderr!r}"
        )
    return STANDALONE_GUARD.resolve()


def parse_json_stdout(stdout: str) -> dict[str, object]:
    stripped = stdout.strip()
    if not stripped:
        raise AssertionError("Guard stdout was empty")
    try:
        value = json.loads(stripped)
    except json.JSONDecodeError as error:
        raise AssertionError(
            f"Guard stdout must contain exactly one JSON object: {stdout!r}"
        ) from error
    if not isinstance(value, dict):
        raise AssertionError(f"Guard stdout was not a JSON object: {stdout!r}")
    return value


def run_guard(
    arguments: list[str],
    *,
    binary: Path | None = None,
    cwd: Path = REPOSITORY_ROOT,
    expected_exit_codes: set[int] = frozenset({0}),
    environment: dict[str, str] | None = None,
) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
    command = guard_process_command(binary)
    completed = subprocess.run(
        [*command, *arguments],
        cwd=cwd,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )
    report = parse_json_stdout(completed.stdout)
    if completed.returncode not in expected_exit_codes:
        raise AssertionError(
            f"Guard exited {completed.returncode}, expected {sorted(expected_exit_codes)}; "
            f"stderr={completed.stderr!r}; report={report!r}"
        )
    if report.get("exit_code") != completed.returncode:
        raise AssertionError(
            "Guard process exit code and report exit_code diverged: "
            f"{completed.returncode!r} != {report.get('exit_code')!r}"
        )
    return completed, report


def guard_process_command(binary: Path | None = None) -> list[str]:
    if binary is not None:
        return [str(binary)]
    if os.environ.get("AW_EC_STAGE") == "td":
        return [
            "uv",
            "run",
            "--frozen",
            "--offline",
            "--project",
            str(TECH_DESIGN_ROOT),
            "python",
            str(TECH_DESIGN_ENTRYPOINT),
        ]
    return [str(guard_binary())]


def expected_static_engine() -> str:
    return (
        "guard-python-reference"
        if os.environ.get("AW_EC_STAGE") == "td"
        else "compass"
    )


def expected_finding_source() -> str:
    return expected_static_engine()


def expected_finding_id_prefix(rule: str) -> str:
    namespace = (
        "guard-reference"
        if os.environ.get("AW_EC_STAGE") == "td"
        else "compass"
    )
    return f"{namespace}:{rule}:"


def assert_report_shape(report: dict[str, object], *, verb: str) -> list[str]:
    required = {
        "schema_version",
        "tool_version",
        "verb",
        "target",
        "policy_profile",
        "status",
        "exit_code",
        "summary",
        "findings",
        "completion",
        "integrations",
        "agent_prompt",
    }
    missing = sorted(required - report.keys())
    if missing:
        raise AssertionError(f"Guard report is missing fields: {missing}")
    if report["schema_version"] != "guard.report/1":
        raise AssertionError(f"unexpected Guard schema: {report['schema_version']!r}")
    if report["verb"] != verb:
        raise AssertionError(f"unexpected Guard verb: {report['verb']!r}")
    if not isinstance(report["status"], dict) or not isinstance(report["summary"], dict):
        raise AssertionError("Guard status and summary must be objects")
    if not isinstance(report["findings"], list):
        raise AssertionError("Guard findings must be a list")
    if not isinstance(report["completion"], dict):
        raise AssertionError("Guard completion must be an object")
    if not isinstance(report["integrations"], dict):
        raise AssertionError("Guard integrations must be an object")
    if not isinstance(report["agent_prompt"], str) or not report["agent_prompt"]:
        raise AssertionError("Guard agent_prompt must be non-empty")
    return [
        "schema_version is guard.report/1",
        f"verb is {verb}",
        "required report fields have stable JSON types",
    ]


def assert_scan_consistency(report: dict[str, object]) -> list[str]:
    assertions = assert_report_shape(report, verb="scan")
    summary = report["summary"]
    findings = report["findings"]
    completion = report["completion"]
    status = report["status"]
    assert isinstance(summary, dict)
    assert isinstance(findings, list)
    assert isinstance(completion, dict)
    assert isinstance(status, dict)
    finding_count = summary.get("security_findings")
    if finding_count != len(findings):
        raise AssertionError(
            f"summary/list finding counts diverged: {finding_count!r} != {len(findings)}"
        )
    expected_clean = report["exit_code"] == 0
    if completion.get("clean") is not expected_clean:
        raise AssertionError("completion.clean does not match Guard exit state")
    if status.get("state") != ("clean" if expected_clean else "findings"):
        raise AssertionError("status.state does not match Guard exit state")
    assertions.extend(
        [
            "summary.security_findings equals findings length",
            "status, exit_code, and completion.clean agree",
        ]
    )
    return assertions


def assert_finding(report: dict[str, object], *, rule: str) -> dict[str, object]:
    findings = report.get("findings")
    if not isinstance(findings, list):
        raise AssertionError("Guard findings must be a list")
    matching = [
        item for item in findings if isinstance(item, dict) and item.get("rule") == rule
    ]
    if len(matching) != 1:
        raise AssertionError(f"expected exactly one {rule} finding, got {matching!r}")
    return matching[0]


@contextmanager
def fixture(files: dict[str, str]) -> Iterator[Path]:
    with tempfile.TemporaryDirectory(prefix="guard-python-ec-") as temp_dir:
        root = Path(temp_dir)
        for relative_path, content in files.items():
            path = root / relative_path
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        yield root


def run_fixture_scan(
    files: dict[str, str],
    *,
    profile: str | None = None,
    expected_exit_codes: set[int] = frozenset({0}),
) -> dict[str, object]:
    with fixture(files) as root:
        arguments = ["scan", str(root)]
        if profile is not None:
            arguments.extend(["--profile", profile])
        arguments.extend(["--compact", "--no-persist"])
        _, report = run_guard(arguments, expected_exit_codes=expected_exit_codes)
        return report


@dataclass(frozen=True)
class AdapterExpectation:
    label: str
    argv: tuple[str, ...]
    report_clean: bool
    exit_code: int
    finding_count: int
    findings_preview: tuple[dict[str, object], ...]

    @property
    def clean(self) -> bool:
        return self.exit_code == 0 and self.report_clean


@dataclass(frozen=True)
class AdapterOutcome:
    report_clean: bool
    exit_code: int
    finding_count: int


def _write_stub(
    binary_dir: Path,
    tool: str,
    *,
    clean: bool = True,
    finding_count: int = 0,
    exit_code: int = 0,
) -> Path:
    trace_path = binary_dir / f"{tool}-argv.txt"
    outcome_path = binary_dir / f"{tool}-outcome.json"
    script = binary_dir / tool
    payload = {
        "schema_version": f"{tool}.report/1",
        "clean": clean,
        "summary": {"total": finding_count},
        "findings": [
            {"id": f"{tool}-finding-{index + 1}"}
            for index in range(finding_count)
        ],
    }
    outcome_path.write_text(
        json.dumps(
            {"payload": payload, "exit_code": exit_code},
            separators=(",", ":"),
        ),
        encoding="utf-8",
    )
    script.symlink_to(ADAPTER_STUB)
    return trace_path


def run_dynamic_adapters(
    tools: tuple[str, ...],
    *,
    outcomes: dict[str, AdapterOutcome] | None = None,
) -> tuple[
    dict[str, object],
    dict[str, list[list[str]]],
    dict[str, AdapterExpectation],
]:
    configured_outcomes = outcomes or {}
    if not set(configured_outcomes).issubset(tools):
        raise AssertionError("adapter outcome must belong to a configured tool")
    DYNAMIC_TMP_ROOT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(
        prefix="dynamic-",
        dir=DYNAMIC_TMP_ROOT,
    ) as temp_dir:
        root = Path(temp_dir)
        source = root / "fixture"
        source.mkdir()
        (source / "safe.js").write_text("const answer = 42;\n", encoding="utf-8")
        scenario = source / "scenario.toml"
        scenario.write_text("name = 'guard-python-ec'\n", encoding="utf-8")
        binary_dir = root / "bin"
        binary_dir.mkdir()
        copied_guard = binary_dir / "guard"
        td_stage = os.environ.get("AW_EC_STAGE") == "td"
        if not td_stage:
            os.link(guard_binary(), copied_guard)
        labels = {
            "vat": "guard-security-smoke",
            "rig": str(scenario),
            "meter": str(source),
        }
        argv = {
            "vat": ("run", "--json", labels["vat"]),
            "rig": ("run", "--scenario", labels["rig"], "--compact"),
            "meter": (
                "run",
                "--target",
                labels["meter"],
                "--skip-bench",
                "--skip-profile",
                "--compact",
            ),
        }
        resolved_outcomes = {
            tool: configured_outcomes.get(tool, AdapterOutcome(True, 0, 0))
            for tool in tools
        }
        expectations = {
            tool: AdapterExpectation(
                label=labels[tool],
                argv=argv[tool],
                report_clean=resolved_outcomes[tool].report_clean,
                exit_code=resolved_outcomes[tool].exit_code,
                finding_count=resolved_outcomes[tool].finding_count,
                findings_preview=tuple(
                    {
                        "id": f"{tool}-finding-{index + 1}",
                    }
                    for index in range(resolved_outcomes[tool].finding_count)
                ),
            )
            for tool in tools
        }
        trace_paths = {
            tool: _write_stub(
                binary_dir,
                tool,
                clean=expectations[tool].report_clean,
                finding_count=expectations[tool].finding_count,
                exit_code=expectations[tool].exit_code,
            )
            for tool in tools
        }

        arguments = ["scan", str(source), "--compact", "--no-persist"]
        if "vat" in tools:
            arguments.extend(["--vat-runner", expectations["vat"].label])
        if "rig" in tools:
            arguments.extend(["--rig-scenario", expectations["rig"].label])
        if "meter" in tools:
            arguments.extend(["--meter-target", expectations["meter"].label])
        expected_guard_exit = (
            {1}
            if any(not expectation.clean for expectation in expectations.values())
            else {0}
        )
        _, report = run_guard(
            arguments,
            binary=None if td_stage else copied_guard,
            expected_exit_codes=expected_guard_exit,
            environment={
                **os.environ,
                "PATH": f"{binary_dir}{os.pathsep}{os.environ.get('PATH', '')}",
            },
        )
        traces = {
            tool: [
                json.loads(line)
                for line in path.read_text(encoding="utf-8").splitlines()
                if line
            ]
            for tool, path in trace_paths.items()
        }
        return report, traces, expectations


def verify_adapter_route(route: str) -> list[str]:
    """Exercise one public adapter flag through a real Guard process."""
    route_tools = {
        "vat-command": "vat",
        "rig-dir": "rig",
        "rig-command": "rig",
        "meter-command": "meter",
        "arena-spec": "arena",
        "arena-command": "arena",
    }
    tool = route_tools.get(route)
    if tool is None:
        raise AssertionError(f"unsupported adapter route: {route}")

    DYNAMIC_TMP_ROOT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(
        prefix=f"{route}-",
        dir=DYNAMIC_TMP_ROOT,
    ) as temp_dir:
        root = Path(temp_dir)
        source = root / "fixture"
        source.mkdir()
        (source / "safe.js").write_text("const answer = 42;\n", encoding="utf-8")
        rig_dir = root / "rig-cases"
        rig_dir.mkdir()
        arena_spec = root / "arena.toml"
        arena_spec.write_text("name = 'guard-python-ec'\n", encoding="utf-8")
        binary_dir = root / "bin"
        binary_dir.mkdir()
        trace_path = _write_stub(binary_dir, tool)
        copied_guard = binary_dir / "guard"
        td_stage = os.environ.get("AW_EC_STAGE") == "td"
        if not td_stage:
            os.link(guard_binary(), copied_guard)

        if route == "vat-command":
            value = "vat command-route"
            expected_trace = ["command-route"]
            expected_command = ["sh", "-c", value]
        elif route == "rig-dir":
            value = str(rig_dir)
            expected_trace = ["run", "--dir", value, "--compact"]
            expected_command = [str(binary_dir / tool), *expected_trace]
        elif route == "rig-command":
            value = "rig command-route"
            expected_trace = ["command-route"]
            expected_command = ["sh", "-c", value]
        elif route == "meter-command":
            value = "meter command-route"
            expected_trace = ["command-route"]
            expected_command = ["sh", "-c", value]
        elif route == "arena-spec":
            value = str(arena_spec)
            expected_trace = ["run", "--spec", value, "--compact"]
            expected_command = [str(binary_dir / tool), *expected_trace]
        else:
            value = "arena command-route"
            expected_trace = ["command-route"]
            expected_command = ["sh", "-c", value]

        _, report = run_guard(
            [
                "scan",
                str(source),
                f"--{route}",
                value,
                "--compact",
                "--no-persist",
            ],
            binary=None if td_stage else copied_guard,
            environment={
                **os.environ,
                "PATH": f"{binary_dir}{os.pathsep}{os.environ.get('PATH', '')}",
            },
        )
        assertions = assert_scan_consistency(report)
        traces = [
            json.loads(line)
            for line in trace_path.read_text(encoding="utf-8").splitlines()
            if line
        ]
        if traces != [expected_trace]:
            raise AssertionError(
                f"{route} executed argv diverged: {traces!r} != {[expected_trace]!r}"
            )
        evidence = report.get("evidence")
        if not isinstance(evidence, list) or len(evidence) != 1:
            raise AssertionError(
                f"{route} must produce exactly one evidence record: {evidence!r}"
            )
        item = evidence[0]
        if not isinstance(item, dict):
            raise AssertionError(f"{route} evidence record must be an object")
        command = item.get("command")
        if (
            not isinstance(command, list)
            or command[1:] != expected_command[1:]
            or Path(str(command[0])).name != Path(expected_command[0]).name
        ):
            raise AssertionError(
                f"{route} folded command diverged: {command!r} != {expected_command!r}"
            )
        if (
            item.get("tool") != tool
            or item.get("label") != value
            or item.get("status") != "clean"
            or item.get("clean") is not True
            or item.get("exit_code") != 0
            or item.get("finding_count") != 0
        ):
            raise AssertionError(f"{route} folded evidence diverged: {item!r}")
        folded_report = item.get("report")
        if (
            not isinstance(folded_report, dict)
            or folded_report.get("schema_version") != f"{tool}.report/1"
            or folded_report.get("clean") is not True
        ):
            raise AssertionError(f"{route} folded report is invalid: {folded_report!r}")
        assertions.append(
            f"--{route} executes exactly once and preserves its public value"
        )
        return assertions


def assert_dynamic_evidence(
    report: dict[str, object],
    traces: dict[str, list[list[str]]],
    *,
    expectations: dict[str, AdapterExpectation],
) -> list[str]:
    assertions = assert_scan_consistency(report)
    evidence = report.get("evidence")
    if not isinstance(evidence, list):
        raise AssertionError("Guard dynamic scan must expose an evidence array")
    if len(evidence) != len(expectations):
        raise AssertionError(
            f"evidence cardinality does not match configured invocations: "
            f"{len(evidence)} != {len(expectations)}"
        )
    by_tool: dict[object, dict[str, object]] = {}
    for item in evidence:
        if not isinstance(item, dict):
            raise AssertionError("every dynamic evidence element must be an object")
        tool = item.get("tool")
        if tool in by_tool:
            raise AssertionError(f"duplicate evidence record for tool {tool!r}")
        by_tool[tool] = item
    if set(by_tool) != set(expectations):
        raise AssertionError(
            f"unexpected evidence tools: {sorted(str(item) for item in by_tool)}"
        )
    summary = report["summary"]
    assert isinstance(summary, dict)
    if summary.get("evidence_count") != len(evidence):
        raise AssertionError("summary.evidence_count does not match evidence array")
    expected_failures = sum(not expectation.clean for expectation in expectations.values())
    actual_failures = sum(item.get("clean") is not True for item in evidence)
    if summary.get("evidence_failed") != expected_failures:
        raise AssertionError("summary.evidence_failed does not match configured outcomes")
    if actual_failures != expected_failures:
        raise AssertionError("evidence clean states do not match configured outcomes")
    for tool, expectation in expectations.items():
        item = by_tool[tool]
        assert isinstance(item, dict)
        expected_status = "clean" if expectation.clean else "findings"
        if (
            item.get("clean") is not expectation.clean
            or item.get("status") != expected_status
        ):
            raise AssertionError(f"{tool} evidence outcome diverged: {item!r}")
        invocations = traces.get(tool)
        if not invocations:
            raise AssertionError(f"{tool} adapter was not actually executed")
        if len(invocations) != 1:
            raise AssertionError(
                f"{tool} adapter must execute exactly once, got {len(invocations)}"
            )
        trace = invocations[0]
        if not all(isinstance(argument, str) for argument in trace):
            raise AssertionError(f"{tool} adapter trace is not a string argv: {trace!r}")
        if tuple(trace) != expectation.argv:
            raise AssertionError(
                f"{tool} executed argv does not match caller input: "
                f"{trace!r} != {expectation.argv!r}"
            )
        command = item.get("command")
        if (
            not isinstance(command, list)
            or not command
            or Path(str(command[0])).name != tool
            or command[1:] != trace
        ):
            raise AssertionError(
                f"{tool} folded command does not match executed argv: "
                f"{command!r} != {trace!r}"
            )
        if item.get("label") != expectation.label:
            raise AssertionError(
                f"{tool} folded label does not match caller input: "
                f"{item.get('label')!r} != {expectation.label!r}"
            )
        if (
            item.get("exit_code") != expectation.exit_code
            or item.get("finding_count") != expectation.finding_count
        ):
            raise AssertionError(f"{tool} folded process result diverged: {item!r}")
        folded_report = item.get("report")
        if not isinstance(folded_report, dict):
            raise AssertionError(f"{tool} folded report is not an object")
        if folded_report.get("schema_version") != f"{tool}.report/1":
            raise AssertionError(f"{tool} folded report schema is invalid")
        if folded_report.get("clean") is not expectation.report_clean:
            raise AssertionError(f"{tool} folded report clean state diverged")
        folded_summary = folded_report.get("summary")
        if (
            not isinstance(folded_summary, dict)
            or folded_summary.get("total") != expectation.finding_count
        ):
            raise AssertionError(f"{tool} folded report summary is invalid")
        findings_preview = folded_report.get("findings_preview")
        expected_preview = list(expectation.findings_preview[:4])
        if findings_preview != expected_preview:
            raise AssertionError(f"{tool} folded findings preview is invalid")
    assertions.extend(
        [
            f"evidence tools are exactly {','.join(expectations)} with no duplicates",
            "each configured adapter executes exactly once with caller-owned argv",
            "folded commands exactly match the recorded adapter invocation",
            "folded adapter reports preserve schema, independent outcomes, and findings",
            "evidence_count and evidence_failed summarize the actual evidence array",
        ]
    )
    return assertions
