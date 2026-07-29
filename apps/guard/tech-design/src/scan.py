"""Executable, correctness-first static scanner for the Guard Python TD.

This reference scanner deliberately favors explicit, auditable rules over
performance. The Rust CB uses Compass for faster AST-backed coverage and must
preserve the externally visible results covered by EC.
"""

from __future__ import annotations

import ast
from dataclasses import dataclass
import json
from pathlib import Path
import re
from typing import Iterable

from evidence import EvidenceCommand, run_evidence_commands
from policy import (
    DiagnosticCategory,
    DiagnosticSeverity,
    PolicyDesign,
    PolicyProfile,
)
from report import Finding, GuardReport, Location, Severity

__aw_artifact_id__ = "artifact:guard/static-security-scan"
__aw_public_contract__ = True
__aw_public_behaviors__ = (
    "compass_backed_diagnostic_scan",
    "json_report_envelope",
    "scan_command_report_projection",
    "stable_static_finding_normalization",
)


@dataclass(frozen=True)
class Diagnostic:
    rule: str
    category: DiagnosticCategory
    severity: DiagnosticSeverity
    message: str
    line: int
    start_col: int
    end_col: int
    language: str


@dataclass(frozen=True)
class ScanOptions:
    profile: PolicyProfile = PolicyProfile.BASELINE_STATIC
    evidence_commands: tuple[EvidenceCommand, ...] = ()
    exclude_patterns: tuple[str, ...] = (
        "__pycache__",
        "node_modules",
        "target",
        ".git",
        ".venv",
        ".guard",
    )


class ScanDesign:
    SUPPORTED_LANGUAGES = (
        "python",
        "typescript",
        "rust",
        "javascript",
        "go",
        "html",
        "css",
        "dockerfile",
        "hcl",
        "yaml",
        "toml",
        "sql",
        "graphql",
    )


LANGUAGE_BY_SUFFIX = {
    ".py": "python",
    ".js": "javascript",
    ".jsx": "javascript",
    ".mjs": "javascript",
    ".ts": "typescript",
    ".tsx": "typescript",
    ".rs": "rust",
    ".go": "go",
    ".html": "html",
    ".css": "css",
    ".hcl": "hcl",
    ".tf": "hcl",
    ".yaml": "yaml",
    ".yml": "yaml",
    ".toml": "toml",
    ".sql": "sql",
    ".graphql": "graphql",
    ".gql": "graphql",
}


def scan_path(
    path: Path,
    options: ScanOptions = ScanOptions(),
) -> GuardReport:
    target = Path(path)
    if not target.exists():
        return GuardReport.tool_error(
            "scan",
            str(target),
            5,
            "scan target does not exist",
        )
    files = list(_source_files(target, options.exclude_patterns))
    diagnostics: list[tuple[Path, Diagnostic]] = []
    for source_path in files:
        try:
            source = source_path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        diagnostics.extend(
            (source_path, diagnostic)
            for diagnostic in _scan_source(source_path, source)
        )
    findings = [
        _finding(source_path, diagnostic, options.profile)
        for source_path, diagnostic in diagnostics
        if PolicyDesign.included_rule(
            options.profile,
            diagnostic.category,
            diagnostic.rule,
        )
    ]
    findings = _apply_waivers(target, findings)
    external_evidence = run_evidence_commands(list(options.evidence_commands))
    for item in external_evidence:
        finding = item.to_guard_finding(str(target))
        if finding is not None:
            findings.append(finding)
    return GuardReport.from_scan(
        str(target),
        options.profile.value,
        len(files),
        len(diagnostics),
        findings,
        [item.to_dict() for item in external_evidence],
    )


def _source_files(
    target: Path,
    exclude_patterns: tuple[str, ...],
) -> Iterable[Path]:
    candidates = [target] if target.is_file() else target.rglob("*")
    for path in candidates:
        if not path.is_file():
            continue
        if any(part in exclude_patterns for part in path.parts):
            continue
        if path.name == "Dockerfile" or path.suffix.lower() in LANGUAGE_BY_SUFFIX:
            yield path


def _language(path: Path) -> str:
    if path.name == "Dockerfile":
        return "dockerfile"
    return LANGUAGE_BY_SUFFIX.get(path.suffix.lower(), "text")


def _scan_source(path: Path, source: str) -> list[Diagnostic]:
    language = _language(path)
    diagnostics: list[Diagnostic] = []
    lines = source.splitlines()
    if language in {"javascript", "typescript"}:
        diagnostics.extend(_javascript_diagnostics(lines, language))
    if language == "python":
        diagnostics.extend(_python_diagnostics(source, lines))
    if language == "rust":
        diagnostics.extend(
            _regex_diagnostics(
                lines,
                "rust",
                (
                    (
                        r"\bunsafe\s*\{",
                        "RS201",
                        DiagnosticCategory.SECURITY,
                        DiagnosticSeverity.INFORMATION,
                        "Unsafe block requires a reviewed memory-safety invariant",
                    ),
                ),
            )
        )
    if language == "dockerfile":
        diagnostics.extend(_dockerfile_diagnostics(lines))
    if language == "hcl":
        diagnostics.extend(_terraform_diagnostics(lines, source))
    if language == "yaml":
        diagnostics.extend(_yaml_diagnostics(path, lines))
    if language in {"python", "javascript", "typescript", "go", "sql"}:
        diagnostics.extend(_sql_injection_diagnostics(lines, language))
    return diagnostics


def _javascript_diagnostics(
    lines: list[str],
    language: str,
) -> list[Diagnostic]:
    return _regex_diagnostics(
        lines,
        language,
        (
            (
                r"\beval\s*\(",
                "JS004",
                DiagnosticCategory.SECURITY,
                DiagnosticSeverity.ERROR,
                "Avoid eval(); it executes arbitrary code",
            ),
            (
                r"\b(?:setTimeout|setInterval)\s*\(\s*[`'\"]",
                "JS006",
                DiagnosticCategory.SECURITY,
                DiagnosticSeverity.ERROR,
                "Pass a function instead of a string to the timer",
            ),
            (
                r"(?:__proto__|\.prototype)\s*=",
                "JS007",
                DiagnosticCategory.STYLE,
                DiagnosticSeverity.WARNING,
                "Avoid prototype mutation surfaces",
            ),
            (
                r"^\s*with\s*\(",
                "JS008",
                DiagnosticCategory.STYLE,
                DiagnosticSeverity.WARNING,
                "Remove with; dynamic scope is unsafe to audit",
            ),
            (
                r"\bas\s+any\b|:\s*any\b",
                "TS102",
                DiagnosticCategory.STYLE,
                DiagnosticSeverity.HINT,
                "Avoid unconstrained any at a security boundary",
            ),
        ),
    )


def _python_diagnostics(source: str, lines: list[str]) -> list[Diagnostic]:
    diagnostics = _regex_diagnostics(
        lines,
        "python",
        (
            (
                r"\beval\s*\(",
                "PY301",
                DiagnosticCategory.SECURITY,
                DiagnosticSeverity.WARNING,
                "Use ast.literal_eval or a constrained parser",
            ),
            (
                r"\bexec\s*\(",
                "PY302",
                DiagnosticCategory.SECURITY,
                DiagnosticSeverity.WARNING,
                "Avoid executing dynamic code",
            ),
            (
                r"\b(?:c?Pickle|pickle)\.(?:load|loads)\s*\(",
                "PY303",
                DiagnosticCategory.SECURITY,
                DiagnosticSeverity.WARNING,
                "Use a safe serialization format for untrusted data",
            ),
        ),
    )
    # Parse subprocess calls so multiline argument lists cannot evade the
    # reference product. Regex remains useful for the language-agnostic rules
    # above; Python's stdlib AST is the correctness oracle for call structure.
    try:
        tree = ast.parse(source)
    except SyntaxError:
        tree = None
    if tree is not None:
        for node in ast.walk(tree):
            if not isinstance(node, ast.Call):
                continue
            function = node.func
            if isinstance(function, ast.Attribute) and isinstance(
                function.value, ast.Name
            ):
                function_name = f"{function.value.id}.{function.attr}"
            elif isinstance(function, ast.Name):
                function_name = function.id
            else:
                continue
            if function_name not in {
                "subprocess.run",
                "subprocess.Popen",
                "subprocess.call",
                "subprocess.check_call",
                "subprocess.check_output",
                "Popen",
                "call",
                "check_call",
                "check_output",
            }:
                continue
            shell_true = any(
                keyword.arg == "shell"
                and isinstance(keyword.value, ast.Constant)
                and keyword.value.value is True
                for keyword in node.keywords
            )
            if shell_true:
                diagnostics.append(
                    Diagnostic(
                        "PY304",
                        DiagnosticCategory.SECURITY,
                        DiagnosticSeverity.WARNING,
                        "Avoid shell=True; pass a validated argv array",
                        node.lineno,
                        node.col_offset + 1,
                        (node.end_col_offset or node.col_offset + 1) + 1,
                        "python",
                    )
                )
    secret_pattern = re.compile(
        r"(?i)\b(password|secret|api_?key|token|private_key|access_key|secret_key)"
        r"\s*=\s*(?:b)?(['\"])(?!\2)"
    )
    for line_number, line in enumerate(lines, start=1):
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        match = secret_pattern.search(line)
        if match and not any(
            source in line
            for source in ("os.environ", "os.getenv", "environ.get")
        ):
            diagnostics.append(
                Diagnostic(
                    "PY305",
                    DiagnosticCategory.SECURITY,
                    DiagnosticSeverity.WARNING,
                    "Move hardcoded secrets to environment or a secrets manager",
                    line_number,
                    match.start() + 1,
                    match.end() + 1,
                    "python",
                )
            )
    return diagnostics


def _dockerfile_diagnostics(lines: list[str]) -> list[Diagnostic]:
    diagnostics: list[Diagnostic] = []
    for line_number, line in enumerate(lines, start=1):
        stripped = line.strip()
        upper = stripped.upper()
        if upper.startswith("FROM "):
            image = stripped.split(maxsplit=1)[1].split(" AS ", maxsplit=1)[0]
            if image.endswith(":latest") or (":" not in image and "@" not in image):
                diagnostics.append(
                    Diagnostic(
                        "DK002",
                        DiagnosticCategory.STYLE,
                        DiagnosticSeverity.WARNING,
                        "Base image has no version or digest",
                        line_number,
                        1,
                        len(line) + 1,
                        "dockerfile",
                    )
                )
        if upper.startswith(("COPY ", "ADD ")) and "--chown" not in stripped:
            diagnostics.append(
                Diagnostic(
                    "DK004",
                    DiagnosticCategory.SECURITY,
                    DiagnosticSeverity.WARNING,
                    "COPY/ADD without --chown leaves files owned by root",
                    line_number,
                    1,
                    len(line) + 1,
                    "dockerfile",
                )
            )
        if upper.startswith(("ENV ", "ARG ")) and re.search(
            r"(?i)(password|secret|token|api_?key|private_key|access_key|credential)\w*\s*=\s*\S+",
            stripped,
        ):
            diagnostics.append(
                Diagnostic(
                    "DK009",
                    DiagnosticCategory.SECURITY,
                    DiagnosticSeverity.ERROR,
                    "Hardcoded Docker build secret",
                    line_number,
                    1,
                    len(line) + 1,
                    "dockerfile",
                )
            )
    return diagnostics


def _terraform_diagnostics(
    lines: list[str],
    source: str,
) -> list[Diagnostic]:
    diagnostics = _regex_diagnostics(
        lines,
        "hcl",
        (
            (
                r"(?i)\b(?:password|secret|token|api_?key|private_key|access_key|credential)\w*\s*=\s*\"(?!\$\{(?:var|data|local)\.)",
                "TF004",
                DiagnosticCategory.SECURITY,
                DiagnosticSeverity.ERROR,
                "Use a Terraform variable or secrets manager",
            ),
        ),
    )
    for match in re.finditer(
        r'resource\s+"aws_s3_bucket"\s+"[^"]+"\s*\{',
        source,
    ):
        block = source[match.start() :]
        end = block.find("\n}")
        block = block if end < 0 else block[:end]
        if "server_side_encryption_configuration" not in block:
            line = source[: match.start()].count("\n") + 1
            diagnostics.append(
                Diagnostic(
                    "TF010",
                    DiagnosticCategory.SECURITY,
                    DiagnosticSeverity.WARNING,
                    "S3 bucket is missing server-side encryption",
                    line,
                    1,
                    2,
                    "hcl",
                )
            )
    return diagnostics


def _yaml_diagnostics(path: Path, lines: list[str]) -> list[Diagnostic]:
    diagnostics: list[Diagnostic] = []
    is_kubernetes = any(line.strip().startswith("apiVersion:") for line in lines)
    if is_kubernetes:
        for line_number, line in enumerate(lines, start=1):
            stripped = line.strip()
            if stripped.startswith("image:"):
                image = stripped.split(":", maxsplit=1)[1].strip().strip("'\"")
                if image.endswith(":latest") or (
                    ":" not in image and "@" not in image
                ):
                    diagnostics.append(
                        Diagnostic(
                            "K8003",
                            DiagnosticCategory.SECURITY,
                            DiagnosticSeverity.WARNING,
                            "Pin container images to a version or digest",
                            line_number,
                            1,
                            len(line) + 1,
                            "yaml",
                        )
                    )
        has_containers = any("containers:" in line for line in lines)
        run_as_non_root = any(
            "runAsNonRoot:" in line and "true" in line.lower() for line in lines
        )
        if has_containers and not run_as_non_root:
            diagnostics.append(
                Diagnostic(
                    "K8006",
                    DiagnosticCategory.SECURITY,
                    DiagnosticSeverity.INFORMATION,
                    "Set securityContext.runAsNonRoot: true",
                    1,
                    1,
                    2,
                    "yaml",
                )
            )
    if path.name == ".gitlab-ci.yml":
        diagnostics.extend(
            _regex_diagnostics(
                lines,
                "yaml",
                (
                    (
                        r"(?i)^\s*(?:password|secret|token|api_?key|private_key|access_key)\w*\s*:\s*['\"]?\S+",
                        "GL008",
                        DiagnosticCategory.SECURITY,
                        DiagnosticSeverity.ERROR,
                        "Use a masked CI variable instead of a hardcoded secret",
                    ),
                ),
            )
        )
    return diagnostics


def _sql_injection_diagnostics(
    lines: list[str],
    language: str,
) -> list[Diagnostic]:
    patterns = (
        r"(?i)(?:select|insert|update|delete).*(?:\{[^}]+\}|\+\s*\w+|%\s*\w+)",
        r"(?i)(?:execute|query)\s*\(\s*f?['\"`].*(?:\{[^}]+\}|\+\s*\w+)",
    )
    return _regex_diagnostics(
        lines,
        language,
        tuple(
            (
                pattern,
                "SQL-INJ",
                DiagnosticCategory.SECURITY,
                DiagnosticSeverity.WARNING,
                "Use parameterized queries and keep untrusted values out of SQL",
            )
            for pattern in patterns
        ),
    )


def _regex_diagnostics(
    lines: list[str],
    language: str,
    rules: tuple[
        tuple[
            str,
            str,
            DiagnosticCategory,
            DiagnosticSeverity,
            str,
        ],
        ...,
    ],
) -> list[Diagnostic]:
    diagnostics: list[Diagnostic] = []
    for line_number, line in enumerate(lines, start=1):
        for pattern, rule, category, severity, message in rules:
            match = re.search(pattern, line)
            if match:
                diagnostics.append(
                    Diagnostic(
                        rule,
                        category,
                        severity,
                        message,
                        line_number,
                        match.start() + 1,
                        match.end() + 1,
                        language,
                    )
                )
    return diagnostics


def _finding(
    path: Path,
    diagnostic: Diagnostic,
    profile: PolicyProfile,
) -> Finding:
    severity = PolicyDesign.map_severity(profile, diagnostic.severity)
    return Finding(
        id=_finding_id(diagnostic.rule, str(path), diagnostic.line),
        severity=severity,
        rule=diagnostic.rule,
        title=diagnostic.message,
        detail=(
            f"reference scanner reported {diagnostic.severity.value} "
            f"{diagnostic.category.value} diagnostic {diagnostic.rule} "
            f"at {path}:{diagnostic.line}"
        ),
        remediation=_remediation_for_rule(diagnostic.rule),
        location=Location(
            str(path),
            diagnostic.line,
            diagnostic.start_col,
            diagnostic.line,
            diagnostic.end_col,
        ),
        evidence={
            "source": "guard-python-reference",
            "reference_engine": "guard-python-td",
            "diagnostic_category": diagnostic.category.value,
            "diagnostic_severity": diagnostic.severity.value,
            "language": diagnostic.language,
        },
    )


def _apply_waivers(target: Path, findings: list[Finding]) -> list[Finding]:
    root = target if target.is_dir() else target.parent
    waiver_path = root / ".guard" / "waivers.json"
    try:
        value = json.loads(waiver_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return findings
    waivers = value.get("waivers", []) if isinstance(value, dict) else []
    if not isinstance(waivers, list):
        return findings
    kept: list[Finding] = []
    for finding in findings:
        waived = any(
            isinstance(waiver, dict)
            and waiver.get("rule") == finding.rule
            and (
                not waiver.get("path_contains")
                or str(waiver["path_contains"]) in finding.location.path
            )
            for waiver in waivers
        )
        if not waived:
            kept.append(finding)
    return kept


def _finding_id(rule: str, path: str, line: int) -> str:
    subject = f"{path}:{line}"
    squashed = "".join(
        character if character.isalnum() or character in "-_" else "-"
        for character in subject
    )
    return f"guard-reference:{rule}:{squashed}"


def _remediation_for_rule(rule: str) -> str:
    return {
        "JS004": "Remove dynamic code execution or replace it with a constrained parser/dispatcher.",
        "JS006": "Pass a function to the timer instead of executable source text.",
        "PY301": "Replace eval with ast.literal_eval or a constrained parser.",
        "PY302": "Remove exec and use an explicit dispatcher.",
        "PY303": "Avoid pickle for untrusted data; use a safe serialization format.",
        "PY304": "Avoid shell=True; pass an argv array and validate untrusted inputs.",
        "PY305": "Move secrets into environment variables or a secrets manager.",
        "RS201": "Document and audit the unsafe invariant; add a focused safety test.",
        "SQL-INJ": "Use parameterized queries and keep untrusted values out of SQL strings.",
        "DK002": "Pin the image by version or digest to avoid supply-chain drift.",
        "DK004": "Use COPY --chown or drop privileges before consuming copied files.",
        "DK009": "Use build secrets or runtime secret injection.",
        "JS007": "Avoid prototype mutation surfaces.",
        "JS008": "Remove with; it creates dynamic scope that is hard to audit.",
        "TF004": "Use a Terraform variable or secrets manager.",
        "TF010": "Configure server-side encryption for the S3 bucket.",
        "K8003": "Pin the image by version or digest.",
        "K8006": "Set securityContext.runAsNonRoot: true.",
        "GL008": "Use a masked CI/CD variable.",
    }.get(
        rule,
        "Inspect the source, remove the risky pattern, or document a reviewed waiver.",
    )


def compass_backed_diagnostic_scan() -> str:
    return scan_path(Path("__missing__")).schema_version


def json_report_envelope() -> str:
    return GuardReport.stub("spec", "contract").schema_version


def scan_command_report_projection() -> tuple[str, str]:
    report = scan_path(Path("__missing__"))
    return report.verb, report.target


def stable_static_finding_normalization() -> str:
    return _finding_id("JS004", "unsafe.js", 1)
