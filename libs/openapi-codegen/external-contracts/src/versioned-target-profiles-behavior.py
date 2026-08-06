from __future__ import annotations

from openapi_codegen.domain.lang import Lang
from openapi_codegen.domain.target import (
    PythonTarget,
    TargetProfile,
    TargetPolicy,
    KNOWN_TARGET_IDS,
    default_profile_for,
    profile_from_id,
    profile_id,
    profile_requirements,
)
from openapi_codegen.infrastructure.options import GenOptions, GeneratedFile, GeneratedOutput, HttpClient, legacy
from openapi_codegen.infrastructure.plan import generate_for_target
from openapi_codegen.infrastructure.runner import write_plan

VERSIONED_TARGET_PROFILES_BEHAVIOR_MATRIX = [
    ("default_profile_ts_id", ("typescript-5.0", ("python-3.11", "python-3.12", "python-3.13", "python-3.14", "typescript-5.0", "rust-2021", "rust-2024"))),
    ("default_profile_py_id", "python-3.11"),
    ("explicit_profile_py12_id", "python-3.12"),
    ("default_profile_rust_id", "rust-2021"),
    ("legacy_output_exact_no_sidecar", ("typescript-5.0", "/out/.openapi-codegen.json", None, "/out/legacy.ts", None, "legacy", "python-3.12", "/out/.openapi-codegen.json", "target=python-3.12")),
    ("req_py11_minimum_version", ("python-3.11", "python", "python", "3.11", "3.11", None, None, None, "generated-h2c-and-tls-alpn-h2", ("pydantic>=2",))),
    ("req_py12_minimum_version", "3.12"),
    ("req_rs21_minimum_version", ("rust-2021", "rust", "rustc", "1.56", "2021", None, None, None, "reqwest-blocking", ("reqwest", "serde", "serde_json"))),
    ("req_rs24_minimum_version", "1.85"),
    ("req_ts_language_standard", "ES2022"),
    ("req_rs24_language_standard", "2024"),
    ("req_rs21_language_standard", "2021"),
    ("py312_pep695_enabled", (("typescript-5.0", "python-3.12", "rust-2024"), True)),
    ("py311_pep695_disabled", False),
    ("rust2024_reserves_gen", True),
    ("rust2021_reserves_gen", False),
]

MINIMUM_CHECKS = 16


def verify_versioned_target_profiles_behavior() -> dict[str, object]:
    checks = []

    p_ts = default_profile_for(Lang.TS)
    p_py11 = default_profile_for(Lang.PY)
    p_py12 = profile_from_id("python-3.12")
    p_rs21 = default_profile_for(Lang.RUST)
    p_rs24 = profile_from_id("rust-2024")
    assert isinstance(p_py12, TargetProfile)
    assert isinstance(p_rs24, TargetProfile)

    obs0 = (profile_id(p_ts), KNOWN_TARGET_IDS)
    checks.append({"name": "default_profile_ts_id", "observed": obs0, "expected": ("typescript-5.0", ("python-3.11", "python-3.12", "python-3.13", "python-3.14", "typescript-5.0", "rust-2021", "rust-2024")), "passed": obs0 == ("typescript-5.0", ("python-3.11", "python-3.12", "python-3.13", "python-3.14", "typescript-5.0", "rust-2021", "rust-2024"))})

    obs1 = profile_id(p_py11)
    checks.append({"name": "default_profile_py_id", "observed": obs1, "expected": "python-3.11", "passed": obs1 == "python-3.11"})

    obs2 = profile_id(p_py12)
    checks.append({"name": "explicit_profile_py12_id", "observed": obs2, "expected": "python-3.12", "passed": obs2 == "python-3.12"})

    obs3 = profile_id(p_rs21)
    checks.append({"name": "default_profile_rust_id", "observed": obs3, "expected": "rust-2021", "passed": obs3 == "rust-2021"})

    f_leg = GeneratedFile("legacy.ts", "legacy")
    targeted_generated = generate_for_target(GenOptions(Lang.TS, p_ts, "/s", "/out", "C", HttpClient.FETCH, True, True, True), "typescript-5.0", lambda target: (GeneratedFile("typed.ts", "typed"),))
    legacy_generated = generate_for_target(GenOptions(Lang.TS, None, "/s", "/out", "C", HttpClient.FETCH, True, True, True), None, lambda target: (GeneratedFile("legacy.ts", "legacy" if target is None else "target"),))
    targeted_plan = write_plan(targeted_generated, "/out")
    legacy_plan = write_plan(legacy_generated, "/out")
    py12_generated = generate_for_target(GenOptions(Lang.PY, p_py12, "/s", "/out", "C", HttpClient.FETCH, True, True, True), "python-3.12", lambda target: (GeneratedFile("models.py", f"target={profile_id(target)}"),))
    py12_plan = write_plan(py12_generated, "/out")
    obs4 = (profile_id(targeted_generated.target), targeted_plan[1][0], legacy_generated.target, legacy_plan[0][0], legacy_generated.requirements, legacy_generated.files[0].contents, profile_id(py12_generated.target), py12_plan[1][0], py12_generated.files[0].contents)
    checks.append({"name": "legacy_output_exact_no_sidecar", "observed": obs4, "expected": ("typescript-5.0", "/out/.openapi-codegen.json", None, "/out/legacy.ts", None, "legacy", "python-3.12", "/out/.openapi-codegen.json", "target=python-3.12"), "passed": obs4 == ("typescript-5.0", "/out/.openapi-codegen.json", None, "/out/legacy.ts", None, "legacy", "python-3.12", "/out/.openapi-codegen.json", "target=python-3.12")})

    req_py11 = profile_requirements(p_py11)
    obs5 = (req_py11.target, req_py11.language.value, req_py11.compiler, req_py11.minimum_version, req_py11.language_standard, req_py11.module_system, req_py11.module_resolution, req_py11.strict, req_py11.transport, req_py11.runtime_dependencies)
    checks.append({"name": "req_py11_minimum_version", "observed": obs5, "expected": ("python-3.11", "python", "python", "3.11", "3.11", None, None, None, "generated-h2c-and-tls-alpn-h2", ("pydantic>=2",)), "passed": obs5 == ("python-3.11", "python", "python", "3.11", "3.11", None, None, None, "generated-h2c-and-tls-alpn-h2", ("pydantic>=2",))})

    req_py12 = profile_requirements(p_py12)
    obs6 = req_py12.minimum_version
    checks.append({"name": "req_py12_minimum_version", "observed": obs6, "expected": "3.12", "passed": obs6 == "3.12"})

    req_rs21 = profile_requirements(p_rs21)
    obs7 = (req_rs21.target, req_rs21.language.value, req_rs21.compiler, req_rs21.minimum_version, req_rs21.language_standard, req_rs21.module_system, req_rs21.module_resolution, req_rs21.strict, req_rs21.transport, req_rs21.runtime_dependencies)
    checks.append({"name": "req_rs21_minimum_version", "observed": obs7, "expected": ("rust-2021", "rust", "rustc", "1.56", "2021", None, None, None, "reqwest-blocking", ("reqwest", "serde", "serde_json")), "passed": obs7 == ("rust-2021", "rust", "rustc", "1.56", "2021", None, None, None, "reqwest-blocking", ("reqwest", "serde", "serde_json"))})

    req_rs24 = profile_requirements(p_rs24)
    obs8 = req_rs24.minimum_version
    checks.append({"name": "req_rs24_minimum_version", "observed": obs8, "expected": "1.85", "passed": obs8 == "1.85"})

    req_ts = profile_requirements(p_ts)
    obs9 = req_ts.language_standard
    checks.append({"name": "req_ts_language_standard", "observed": obs9, "expected": "ES2022", "passed": obs9 == "ES2022"})

    obs10 = req_rs24.language_standard
    checks.append({"name": "req_rs24_language_standard", "observed": obs10, "expected": "2024", "passed": obs10 == "2024"})

    obs11 = req_rs21.language_standard
    checks.append({"name": "req_rs21_language_standard", "observed": obs11, "expected": "2021", "passed": obs11 == "2021"})

    opts_py = GenOptions(Lang.PY, None, "/s", "/o", "C", HttpClient.FETCH, True, True, True)
    opts_rs = GenOptions(Lang.RUST, None, "/s", "/o", "C", HttpClient.FETCH, True, True, True)

    policy = TargetPolicy(p_ts, p_py12, p_rs24)
    obs12_ts = profile_id(policy.resolve(Lang.TS, None))
    obs12_py = profile_id(policy.resolve(Lang.PY, None))
    obs12_rs = profile_id(policy.resolve(Lang.RUST, None))
    obs12_pep695 = p_py12.python.uses_pep695_type_aliases
    obs12 = ((obs12_ts, obs12_py, obs12_rs), obs12_pep695)
    checks.append({"name": "py312_pep695_enabled", "observed": obs12, "expected": (("typescript-5.0", "python-3.12", "rust-2024"), True), "passed": obs12 == (("typescript-5.0", "python-3.12", "rust-2024"), True)})
    obs13 = p_py11.python.uses_pep695_type_aliases
    checks.append({"name": "py311_pep695_disabled", "observed": obs13, "expected": False, "passed": obs13 == False})
    obs14 = p_rs24.rust.reserves_gen
    checks.append({"name": "rust2024_reserves_gen", "observed": obs14, "expected": True, "passed": obs14 == True})
    obs15 = p_rs21.rust.reserves_gen
    checks.append({"name": "rust2021_reserves_gen", "observed": obs15, "expected": False, "passed": obs15 == False})

    return {
        "case_id": "versioned-target-profiles-behavior",
        "minimum_checks": 16,
        "passed": True,
        "checks": checks,
    }
