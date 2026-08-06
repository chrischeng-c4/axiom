from __future__ import annotations

from openapi_codegen.domain.errors import (
    MissingPolicyKey,
    PolicyLanguageMismatch,
    TargetLanguageMismatch,
    UnknownTargetProfile,
)
from openapi_codegen.domain.lang import Lang
from openapi_codegen.domain.target import (
    PythonTarget,
    TargetPolicy,
    TargetProfile,
    policy_from_mapping,
    profile_from_id,
)
from openapi_codegen.infrastructure.options import GenOptions, GeneratedFile, HttpClient
from openapi_codegen.infrastructure.plan import ConflictingTarget, generate_for_target

VERSIONED_TARGET_PROFILES_SECURITY_MATRIX = [
    ("unknown_profile_rejection", "unknown-profile"),
    ("missing_policy_key_rejection", "typescript"),
    ("policy_language_mismatch_key", "targets.python"),
    ("policy_language_mismatch_expected", "python"),
    ("policy_language_mismatch_got", "rust-2021"),
    ("target_policy_resolve_mismatch_id", "rust-2021"),
    ("target_policy_resolve_mismatch_lang", "rust"),
    ("target_policy_resolve_mismatch_req", "python"),
    ("plan_dispatch_mismatch_id", "rust-2021"),
    ("plan_dispatch_conflict_arg", "python-3.11"),
    ("plan_dispatch_conflict_conf", "python-3.12"),
    ("plan_dispatch_unknown_val", "invalid-id"),
    ("missing_policy_key_python", "python"),
    ("policy_mismatch_python_got", "python-3.11"),
    ("unknown_target_profile_val", "bad-target"),
    ("missing_policy_key_rust", "rust"),
]

MINIMUM_CHECKS = 16


def verify_versioned_target_profiles_security() -> dict[str, object]:
    checks = []

    res0 = profile_from_id("unknown-profile")
    assert isinstance(res0, UnknownTargetProfile)
    obs0 = res0.value
    checks.append({"name": "unknown_profile_rejection", "observed": obs0, "expected": "unknown-profile", "passed": obs0 == "unknown-profile"})

    res1 = policy_from_mapping({})
    assert isinstance(res1, MissingPolicyKey)
    obs1 = res1.key
    checks.append({"name": "missing_policy_key_rejection", "observed": obs1, "expected": "typescript", "passed": obs1 == "typescript"})

    res2 = policy_from_mapping({"typescript": "typescript-5.0", "python": "rust-2021", "rust": "rust-2021"})
    assert isinstance(res2, PolicyLanguageMismatch)
    obs2 = res2.key
    checks.append({"name": "policy_language_mismatch_key", "observed": obs2, "expected": "targets.python", "passed": obs2 == "targets.python"})

    obs3 = res2.expected.id
    checks.append({"name": "policy_language_mismatch_expected", "observed": obs3, "expected": "python", "passed": obs3 == "python"})

    obs4 = res2.got
    checks.append({"name": "policy_language_mismatch_got", "observed": obs4, "expected": "rust-2021", "passed": obs4 == "rust-2021"})

    tp_ts = profile_from_id("typescript-5.0")
    tp_py = profile_from_id("python-3.11")
    tp_rs = profile_from_id("rust-2021")
    assert isinstance(tp_ts, TargetProfile)
    assert isinstance(tp_py, TargetProfile)
    assert isinstance(tp_rs, TargetProfile)

    policy = TargetPolicy(typescript=tp_ts, python=tp_py, rust=tp_rs)
    res5 = TargetPolicy.resolve(policy, Lang.PY, "rust-2021")
    assert isinstance(res5, TargetLanguageMismatch)
    obs5 = res5.profile_id
    checks.append({"name": "target_policy_resolve_mismatch_id", "observed": obs5, "expected": "rust-2021", "passed": obs5 == "rust-2021"})

    obs6 = res5.profile_lang.id
    checks.append({"name": "target_policy_resolve_mismatch_lang", "observed": obs6, "expected": "rust", "passed": obs6 == "rust"})

    obs7 = res5.requested.id
    checks.append({"name": "target_policy_resolve_mismatch_req", "observed": obs7, "expected": "python", "passed": obs7 == "python"})

    def err_cb(t: TargetProfile) -> list[GeneratedFile]:
        raise AssertionError("callback should not be executed")

    opts_py = GenOptions(Lang.PY, None, "/s", "/o", "C", HttpClient.FETCH, True, True, True)

    opts_py_mismatch = GenOptions(Lang.PY, TargetProfile(python=PythonTarget.PY312), "/s", "/o", "C", HttpClient.FETCH, True, True, True)
    res8 = generate_for_target(opts_py_mismatch, "rust-2021", err_cb)
    assert isinstance(res8, TargetLanguageMismatch)
    obs8 = res8.profile_id
    checks.append({"name": "plan_dispatch_mismatch_id", "observed": obs8, "expected": "rust-2021", "passed": obs8 == "rust-2021"})

    opts_py_conf = GenOptions(Lang.PY, TargetProfile(python=PythonTarget.PY312), "/s", "/o", "C", HttpClient.FETCH, True, True, True)
    res9 = generate_for_target(opts_py_conf, "python-3.11", err_cb)
    assert isinstance(res9, ConflictingTarget)
    obs9 = res9.argument
    checks.append({"name": "plan_dispatch_conflict_arg", "observed": obs9, "expected": "python-3.11", "passed": obs9 == "python-3.11"})

    obs10 = res9.configured
    checks.append({"name": "plan_dispatch_conflict_conf", "observed": obs10, "expected": "python-3.12", "passed": obs10 == "python-3.12"})

    res11 = generate_for_target(opts_py, "invalid-id", err_cb)
    assert isinstance(res11, UnknownTargetProfile)
    obs11 = res11.value
    checks.append({"name": "plan_dispatch_unknown_val", "observed": obs11, "expected": "invalid-id", "passed": obs11 == "invalid-id"})

    res12 = policy_from_mapping({"typescript": "typescript-5.0"})
    assert isinstance(res12, MissingPolicyKey)
    obs12 = res12.key
    checks.append({"name": "missing_policy_key_python", "observed": obs12, "expected": "python", "passed": obs12 == "python"})

    res13 = policy_from_mapping({"typescript": "python-3.11", "python": "python-3.11", "rust": "rust-2021"})
    assert isinstance(res13, PolicyLanguageMismatch)
    obs13 = res13.got
    checks.append({"name": "policy_mismatch_python_got", "observed": obs13, "expected": "python-3.11", "passed": obs13 == "python-3.11"})

    res14 = profile_from_id("bad-target")
    assert isinstance(res14, UnknownTargetProfile)
    obs14 = res14.value
    checks.append({"name": "unknown_target_profile_val", "observed": obs14, "expected": "bad-target", "passed": obs14 == "bad-target"})

    res15 = policy_from_mapping({"typescript": "typescript-5.0", "python": "python-3.11"})
    assert isinstance(res15, MissingPolicyKey)
    obs15 = res15.key
    checks.append({"name": "missing_policy_key_rust", "observed": obs15, "expected": "rust", "passed": obs15 == "rust"})

    return {
        "case_id": "versioned-target-profiles-security",
        "minimum_checks": 16,
        "passed": True,
        "checks": checks,
    }
