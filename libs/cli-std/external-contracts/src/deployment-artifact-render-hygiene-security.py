from __future__ import annotations

from cli_std.domain.artifact_render import (
    ensure_trailing_newline,
    release_tag,
    strip_source_ownership_markers,
)

MINIMUM_CHECKS = 11

DEPLOYMENT_ARTIFACT_RENDER_HYGIENE_SECURITY_MATRIX = [
    ("strip_markers_indented_spec_managed_stripped", "line\n"),
    ("strip_markers_codegen_begin_and_end_stripped", "body\n"),
    ("strip_markers_all_marker_file_renders_empty", ""),
    ("strip_markers_empty_input_returns_empty", ""),
    ("ensure_trailing_newline_idempotent_on_newline", "line\n"),
    ("ensure_trailing_newline_appends_when_absent", "line\n"),
    ("release_tag_blank_version_uses_fallback", "mytool@0.1.0"),
    ("release_tag_trims_whitespace", "mytool@1.0.0"),
    ("release_tag_already_qualified_version", "mytool@1.0.0"),
    ("strip_markers_retains_trailing_newline", "a\nb\n"),
    ("release_tag_empty_string_version_uses_fallback", "mytool@0.2.0"),
]


def verify_deployment_artifact_render_hygiene_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    c0 = strip_source_ownership_markers("  # SPEC-MANAGED: test\nline\n")
    checks.append({"name": "strip_markers_indented_spec_managed_stripped", "passed": c0 == "line\n"})

    c1 = strip_source_ownership_markers("# CODEGEN-BEGIN\nbody\n# CODEGEN-END\n")
    checks.append({"name": "strip_markers_codegen_begin_and_end_stripped", "passed": c1 == "body\n"})

    c2 = strip_source_ownership_markers("# SPEC-MANAGED: test\n# CODEGEN-BEGIN\n# CODEGEN-END\n")
    checks.append({"name": "strip_markers_all_marker_file_renders_empty", "passed": c2 == ""})

    c3 = strip_source_ownership_markers("")
    checks.append({"name": "strip_markers_empty_input_returns_empty", "passed": c3 == ""})

    c4 = ensure_trailing_newline("line\n")
    checks.append({"name": "ensure_trailing_newline_idempotent_on_newline", "passed": c4 == "line\n"})

    c5 = ensure_trailing_newline("line")
    checks.append({"name": "ensure_trailing_newline_appends_when_absent", "passed": c5 == "line\n"})

    c6 = release_tag("mytool", "   ", "0.1.0")
    checks.append({"name": "release_tag_blank_version_uses_fallback", "passed": c6 == "mytool@0.1.0"})

    c7 = release_tag("mytool", " 1.0.0 ", "0.1.0")
    checks.append({"name": "release_tag_trims_whitespace", "passed": c7 == "mytool@1.0.0"})

    c8 = release_tag("mytool", "mytool@1.0.0", "0.1.0")
    checks.append({"name": "release_tag_already_qualified_version", "passed": c8 == "mytool@1.0.0"})

    c9 = strip_source_ownership_markers("a\n# CODEGEN-END\nb\n")
    checks.append({"name": "strip_markers_retains_trailing_newline", "passed": c9 == "a\nb\n"})

    c10 = release_tag("mytool", "", "0.2.0")
    checks.append({"name": "release_tag_empty_string_version_uses_fallback", "passed": c10 == "mytool@0.2.0"})

    return {
        "case_id": "deployment-artifact-render-hygiene-security",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
