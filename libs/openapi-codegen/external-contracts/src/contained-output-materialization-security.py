from __future__ import annotations

from openapi_codegen.domain.lang import Lang
from openapi_codegen.domain.output_path import OutputPathEscape, check_output_path, joined_output_path
from openapi_codegen.domain.target import default_profile_for
from openapi_codegen.infrastructure.options import GenOptions, GeneratedFile, GeneratedOutput, HttpClient, for_target, legacy
from openapi_codegen.infrastructure.runner import run, write_plan

CONTAINED_OUTPUT_MATERIALIZATION_SECURITY_MATRIX = [
    ("check_output_path_absolute", ("absolute", "absolute", "absolute", "absolute", "absolute", "absolute")),
    ("check_output_path_parent", ("parent-component", "parent-component", "parent-component", "parent-component")),
    ("joined_output_path_absolute", "absolute"),
    ("joined_output_path_parent", "parent-component"),
    ("write_plan_absolute_escape", ("absolute", "absolute", "absolute", "absolute")),
    ("write_plan_parent_escape", ("parent-component", "parent-component")),
    ("safe_then_escape_exit_code", 1),
    ("absolute_path_exit_code", (1, 1, 1, 1, 1)),
    ("parent_path_exit_code", 1),
    ("spec_read_failure_exit_code", 2),
    ("spec_read_failure_stderr", "openapi-codegen: cannot read /spec.json"),
    ("generation_failure_exit_code", 1),
    ("generation_failure_stderr", "openapi-codegen: failed generation"),
    ("path_escape_stderr_msg", "openapi-codegen: cannot write generated output to /out: generated file path must stay under output directory: 'safe/../../escape.ts' (parent-component)"),
    ("targeted_manifest_stdout_second", ("generated /out/.openapi-codegen.json", (('/out/safe.ts', 'good'), ('/out/.openapi-codegen.json', '{\n  "schema_version": 1,\n  "generator": "openapi-codegen",\n  "compiler": "typescript",\n  "target": "typescript-5.0",\n  "language": "typescript",\n  "minimum_version": "5.0",\n  "language_standard": "ES2022",\n  "module_system": "ESNext",\n  "module_resolution": "Bundler",\n  "strict": true,\n  "transport": "fetch-or-axios",\n  "runtime_dependencies": []\n}\n')))),
    ("legacy_output_stdout_first", ("generated /out/safe.ts", (('/out/safe.ts', 'good'),), ("safe", "client.ts"), ("safe", "mixed", "client.ts"))),
]

MINIMUM_CHECKS = 16


class NoWriteFS:
    def __init__(self, spec: str | None = "{}") -> None:
        self.spec = spec

    def read_text(self, path: str) -> str | None:
        return self.spec

    def write_text(self, path: str, contents: str) -> None:
        raise AssertionError(f"unexpected write to {path}")


class TargetedFS:
    def __init__(self) -> None:
        self.write_log: list[tuple[str, str]] = []

    def read_text(self, path: str) -> str | None:
        return "{}"

    def write_text(self, path: str, contents: str) -> None:
        self.write_log += [(path, contents)]


class LegacyFS:
    def __init__(self) -> None:
        self.write_log: list[tuple[str, str]] = []

    def read_text(self, path: str) -> str | None:
        return "{}"

    def write_text(self, path: str, contents: str) -> None:
        self.write_log += [(path, contents)]


def verify_contained_output_materialization_security() -> dict[str, object]:
    checks = []

    res0 = check_output_path("/etc/passwd")
    assert isinstance(res0, OutputPathEscape)
    res0_drive = check_output_path("C:/escape.ts")
    res0_backslash = check_output_path("C:\\escape.ts")
    res0_unc = check_output_path("\\\\server\\share\\escape.ts")
    assert isinstance(res0_drive, OutputPathEscape) and isinstance(res0_backslash, OutputPathEscape) and isinstance(res0_unc, OutputPathEscape)
    res0_root = check_output_path("\\escape.ts")
    res0_drive_relative = check_output_path("D:escape.ts")
    assert isinstance(res0_root, OutputPathEscape) and isinstance(res0_drive_relative, OutputPathEscape)
    obs0 = (res0.reason, res0_drive.reason, res0_backslash.reason, res0_unc.reason, res0_root.reason, res0_drive_relative.reason)
    checks.append({"name": "check_output_path_absolute", "observed": obs0, "expected": ("absolute", "absolute", "absolute", "absolute", "absolute", "absolute"), "passed": obs0 == ("absolute", "absolute", "absolute", "absolute", "absolute", "absolute")})

    res1 = (check_output_path("../foo"), check_output_path("safe/../../escape.ts"))
    assert all(isinstance(x, OutputPathEscape) for x in res1)
    res1_backslash = check_output_path("safe\\..\\..\\escape.ts")
    res1_mixed = check_output_path("safe/..\\../escape.ts")
    assert isinstance(res1_backslash, OutputPathEscape) and isinstance(res1_mixed, OutputPathEscape)
    obs1 = (res1[0].reason, res1[1].reason, res1_backslash.reason, res1_mixed.reason)
    checks.append({"name": "check_output_path_parent", "observed": obs1, "expected": ("parent-component", "parent-component", "parent-component", "parent-component"), "passed": obs1 == ("parent-component", "parent-component", "parent-component", "parent-component")})

    res2 = joined_output_path("/out", "/etc/passwd")
    assert isinstance(res2, OutputPathEscape)
    obs2 = res2.reason
    checks.append({"name": "joined_output_path_absolute", "observed": obs2, "expected": "absolute", "passed": obs2 == "absolute"})

    res3 = joined_output_path("/out", "../foo")
    assert isinstance(res3, OutputPathEscape)
    obs3 = res3.reason
    checks.append({"name": "joined_output_path_parent", "observed": obs3, "expected": "parent-component", "passed": obs3 == "parent-component"})

    f_abs = GeneratedFile("/etc/passwd", "bad")
    res4 = write_plan(legacy((f_abs,)), "/out")
    assert isinstance(res4, OutputPathEscape)
    abs_plan_drive = write_plan(legacy((GeneratedFile("C:/escape.ts", "bad"),)), "/out")
    abs_plan_backslash = write_plan(legacy((GeneratedFile("C:\\escape.ts", "bad"),)), "/out")
    assert isinstance(abs_plan_drive, OutputPathEscape) and isinstance(abs_plan_backslash, OutputPathEscape)
    abs_plan_root = write_plan(legacy((GeneratedFile("\\escape.ts", "bad"),)), "/out")
    assert isinstance(abs_plan_root, OutputPathEscape)
    obs4 = (res4.reason, abs_plan_drive.reason, abs_plan_backslash.reason, abs_plan_root.reason)
    checks.append({"name": "write_plan_absolute_escape", "observed": obs4, "expected": ("absolute", "absolute", "absolute", "absolute"), "passed": obs4 == ("absolute", "absolute", "absolute", "absolute")})

    f_par = GeneratedFile("../foo", "bad")
    f_nested = GeneratedFile("safe/../../escape.ts", "bad")
    res5 = (write_plan(legacy((f_par,)), "/out"), write_plan(legacy((f_nested,)), "/out"))
    assert all(isinstance(x, OutputPathEscape) for x in res5)
    obs5 = (res5[0].reason, res5[1].reason)
    checks.append({"name": "write_plan_parent_escape", "observed": obs5, "expected": ("parent-component", "parent-component"), "passed": obs5 == ("parent-component", "parent-component")})

    opts = GenOptions(Lang.TS, None, "/spec.json", "/out", "Client", HttpClient.FETCH, True, True, True)

    f_safe = GeneratedFile("safe.ts", "good")
    f_esc = GeneratedFile("safe/../../escape.ts", "bad")
    res6 = run(opts, NoWriteFS(), lambda s, o: legacy((f_safe, f_esc)))
    obs6 = res6.exit_code
    checks.append({"name": "safe_then_escape_exit_code", "observed": obs6, "expected": 1, "passed": obs6 == 1})

    res7 = run(opts, NoWriteFS(), lambda s, o: legacy((f_abs,)))
    res7_drive = run(opts, NoWriteFS(), lambda s, o: legacy((GeneratedFile("C:/escape.ts", "bad"),)))
    res7_unc = run(opts, NoWriteFS(), lambda s, o: legacy((GeneratedFile("\\\\server\\share\\escape.ts", "bad"),)))
    res7_root = run(opts, NoWriteFS(), lambda s, o: legacy((GeneratedFile("\\escape.ts", "bad"),)))
    res7_drive_relative = run(opts, NoWriteFS(), lambda s, o: legacy((GeneratedFile("D:escape.ts", "bad"),)))
    obs7 = (res7.exit_code, res7_drive.exit_code, res7_unc.exit_code, res7_root.exit_code, res7_drive_relative.exit_code)
    checks.append({"name": "absolute_path_exit_code", "observed": obs7, "expected": (1, 1, 1, 1, 1), "passed": obs7 == (1, 1, 1, 1, 1)})

    res8 = run(opts, NoWriteFS(), lambda s, o: legacy((f_par,)))
    obs8 = res8.exit_code
    checks.append({"name": "parent_path_exit_code", "observed": obs8, "expected": 1, "passed": obs8 == 1})

    res9 = run(opts, NoWriteFS(None), lambda s, o: legacy(()))
    obs9 = res9.exit_code
    checks.append({"name": "spec_read_failure_exit_code", "observed": obs9, "expected": 2, "passed": obs9 == 2})

    obs10 = res9.stderr[0]
    checks.append({"name": "spec_read_failure_stderr", "observed": obs10, "expected": "openapi-codegen: cannot read /spec.json", "passed": obs10 == "openapi-codegen: cannot read /spec.json"})

    res11 = run(opts, NoWriteFS("{}"), lambda s, o: "failed generation")
    obs11 = res11.exit_code
    checks.append({"name": "generation_failure_exit_code", "observed": obs11, "expected": 1, "passed": obs11 == 1})

    obs12 = res11.stderr[0]
    checks.append({"name": "generation_failure_stderr", "observed": obs12, "expected": "openapi-codegen: failed generation", "passed": obs12 == "openapi-codegen: failed generation"})

    obs13 = res6.stderr[0]
    checks.append({"name": "path_escape_stderr_msg", "observed": obs13, "expected": "openapi-codegen: cannot write generated output to /out: generated file path must stay under output directory: 'safe/../../escape.ts' (parent-component)", "passed": obs13 == "openapi-codegen: cannot write generated output to /out: generated file path must stay under output directory: 'safe/../../escape.ts' (parent-component)"})

    prof = default_profile_for(Lang.TS)
    opts_tgt = GenOptions(Lang.TS, prof, "/spec.json", "/out", "Client", HttpClient.FETCH, True, True, True)
    targeted_fs = TargetedFS()
    res14 = run(opts_tgt, targeted_fs, lambda s, o: for_target([f_safe], prof))
    targeted_log = tuple(targeted_fs.write_log)
    obs14 = (res14.stdout[1], targeted_log)
    checks.append({"name": "targeted_manifest_stdout_second", "observed": obs14, "expected": ("generated /out/.openapi-codegen.json", (('/out/safe.ts', 'good'), ('/out/.openapi-codegen.json', '{\n  "schema_version": 1,\n  "generator": "openapi-codegen",\n  "compiler": "typescript",\n  "target": "typescript-5.0",\n  "language": "typescript",\n  "minimum_version": "5.0",\n  "language_standard": "ES2022",\n  "module_system": "ESNext",\n  "module_resolution": "Bundler",\n  "strict": true,\n  "transport": "fetch-or-axios",\n  "runtime_dependencies": []\n}\n'))), "passed": obs14 == ("generated /out/.openapi-codegen.json", (('/out/safe.ts', 'good'), ('/out/.openapi-codegen.json', '{\n  "schema_version": 1,\n  "generator": "openapi-codegen",\n  "compiler": "typescript",\n  "target": "typescript-5.0",\n  "language": "typescript",\n  "minimum_version": "5.0",\n  "language_standard": "ES2022",\n  "module_system": "ESNext",\n  "module_resolution": "Bundler",\n  "strict": true,\n  "transport": "fetch-or-axios",\n  "runtime_dependencies": []\n}\n')))} )

    legacy_fs = LegacyFS()
    res15 = run(opts, legacy_fs, lambda s, o: legacy((f_safe,)))
    legacy_log = tuple(legacy_fs.write_log)
    safe_backslash = check_output_path("safe\\client.ts")
    safe_mixed = check_output_path("safe/mixed\\client.ts")
    obs15 = (res15.stdout[0], legacy_log, safe_backslash, safe_mixed)
    checks.append({"name": "legacy_output_stdout_first", "observed": obs15, "expected": ("generated /out/safe.ts", (('/out/safe.ts', 'good'),), ("safe", "client.ts"), ("safe", "mixed", "client.ts")), "passed": obs15 == ("generated /out/safe.ts", (('/out/safe.ts', 'good'),), ("safe", "client.ts"), ("safe", "mixed", "client.ts"))})

    return {
        "case_id": "contained-output-materialization-security",
        "minimum_checks": 16,
        "passed": True,
        "checks": checks,
    }
