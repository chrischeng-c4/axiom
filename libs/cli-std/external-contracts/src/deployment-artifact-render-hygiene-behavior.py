from __future__ import annotations

from cli_std.application.artifact_output import FileOutput, StdoutOutput, has_extension, plan_output
from cli_std.domain.artifact_render import (
    release_tag,
    replace_kubernetes_namespace,
)

MINIMUM_CHECKS = 12

DEPLOYMENT_ARTIFACT_RENDER_HYGIENE_BEHAVIOR_MATRIX = [
    ("release_tag_bare_version", "mytool@1.0.0"),
    ("release_tag_prefixed_version_no_double_prefix", "mytool@1.0.0"),
    ("release_tag_fallback_when_none", "mytool@0.1.0"),
    ("replace_kubernetes_namespace_rewrites_name_and_namespace_fields", "name: new_ns\nnamespace: new_ns\nimage: old_ns/app"),
    ("has_extension_dotted_directory_with_extensionless_leaf", False),
    ("has_extension_dotfile_not_treated_as_extension", False),
    ("has_extension_parent_directory_not_treated_as_file", False),
    ("plan_output_none_resolves_to_stdout", {"type": "stdout", "body": "bytes"}),
    ("plan_output_file_with_extension_resolves_to_file", {"type": "file", "path": "out.yaml", "body": "bytes"}),
    ("plan_output_directory_resolves_to_directory_plus_default", {"type": "file", "path": "out_dir/def.yaml", "body": "bytes"}),
    ("plan_output_trailing_slash_directory_no_double_slash", {"type": "file", "path": "out_dir/def.yaml", "body": "bytes"}),
    ("plan_output_empty_directory_resolves_to_default_name", {"type": "file", "path": "def.yaml", "body": "bytes"}),
]


def verify_deployment_artifact_render_hygiene_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    c0 = release_tag("mytool", "1.0.0", "0.1.0")
    checks.append({"name": "release_tag_bare_version", "passed": c0 == "mytool@1.0.0"})

    c1 = release_tag("mytool", "mytool@1.0.0", "0.1.0")
    checks.append({"name": "release_tag_prefixed_version_no_double_prefix", "passed": c1 == "mytool@1.0.0"})

    c2 = release_tag("mytool", None, "0.1.0")
    checks.append({"name": "release_tag_fallback_when_none", "passed": c2 == "mytool@0.1.0"})

    c3 = replace_kubernetes_namespace("name: old_ns\nnamespace: old_ns\nimage: old_ns/app", "old_ns", "new_ns")
    expected_ns_replace = "name: new_ns\nnamespace: new_ns\nimage: old_ns/app"
    checks.append({"name": "replace_kubernetes_namespace_rewrites_name_and_namespace_fields", "passed": c3 == expected_ns_replace})

    c4 = has_extension("my.dir/manifest")
    checks.append({"name": "has_extension_dotted_directory_with_extensionless_leaf", "passed": c4 == False})

    c5 = has_extension(".env")
    checks.append({"name": "has_extension_dotfile_not_treated_as_extension", "passed": c5 == False})

    c6 = has_extension("dir/..")
    checks.append({"name": "has_extension_parent_directory_not_treated_as_file", "passed": c6 == False})

    res7 = plan_output(None, "def.yaml", "bytes")
    c7 = {"type": "stdout", "body": res7.body} if isinstance(res7, StdoutOutput) else ({"type": "file", "path": res7.path, "body": res7.body} if isinstance(res7, FileOutput) else None)
    checks.append({"name": "plan_output_none_resolves_to_stdout", "passed": c7 == {"type": "stdout", "body": "bytes"}})

    res8 = plan_output("out.yaml", "def.yaml", "bytes")
    c8 = {"type": "stdout", "body": res8.body} if isinstance(res8, StdoutOutput) else ({"type": "file", "path": res8.path, "body": res8.body} if isinstance(res8, FileOutput) else None)
    checks.append({"name": "plan_output_file_with_extension_resolves_to_file", "passed": c8 == {"type": "file", "path": "out.yaml", "body": "bytes"}})

    res9 = plan_output("out_dir", "def.yaml", "bytes")
    c9 = {"type": "stdout", "body": res9.body} if isinstance(res9, StdoutOutput) else ({"type": "file", "path": res9.path, "body": res9.body} if isinstance(res9, FileOutput) else None)
    checks.append({"name": "plan_output_directory_resolves_to_directory_plus_default", "passed": c9 == {"type": "file", "path": "out_dir/def.yaml", "body": "bytes"}})

    res10 = plan_output("out_dir/", "def.yaml", "bytes")
    c10 = {"type": "stdout", "body": res10.body} if isinstance(res10, StdoutOutput) else ({"type": "file", "path": res10.path, "body": res10.body} if isinstance(res10, FileOutput) else None)
    checks.append({"name": "plan_output_trailing_slash_directory_no_double_slash", "passed": c10 == {"type": "file", "path": "out_dir/def.yaml", "body": "bytes"}})

    res11 = plan_output("", "def.yaml", "bytes")
    c11 = {"type": "stdout", "body": res11.body} if isinstance(res11, StdoutOutput) else ({"type": "file", "path": res11.path, "body": res11.body} if isinstance(res11, FileOutput) else None)
    checks.append({"name": "plan_output_empty_directory_resolves_to_default_name", "passed": c11 == {"type": "file", "path": "def.yaml", "body": "bytes"}})

    return {
        "case_id": "deployment-artifact-render-hygiene-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
