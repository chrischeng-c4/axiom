from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.domain.errors import (
    MissingPolicyKey,
    PolicyLanguageMismatch,
    TargetLanguageMismatch,
    UnknownTargetProfile,
)
from openapi_codegen.domain.lang import Lang
from openapi_codegen.domain.target import (
    KNOWN_TARGET_IDS,
    POLICY_KEYS,
    PythonTarget,
    RustTarget,
    TargetPolicy,
    TargetProfile,
    TypeScriptTarget,
    default_profile_for,
    policy_from_mapping,
    profile_from_id,
    profile_id,
    profile_lang,
    profile_requirements,
)


class TestDomainTarget(unittest.TestCase):
    def test_target_profile_zero_variants_raises(self) -> None:
        with self.assertRaises(ValueError) as ctx:
            TargetProfile()
        self.assertEqual(
            str(ctx.exception),
            "TargetProfile must inhabit exactly one variant, got none",
        )

    def test_target_profile_multiple_variants_raises(self) -> None:
        with self.assertRaises(ValueError) as ctx:
            TargetProfile(
                python=PythonTarget.PY311, rust=RustTarget.RUST2024
            )
        self.assertEqual(
            str(ctx.exception),
            "TargetProfile must inhabit exactly one variant, got python, rust",
        )

    def test_target_profile_all_three_variants_raises(self) -> None:
        with self.assertRaises(ValueError) as ctx:
            TargetProfile(
                python=PythonTarget.PY311,
                typescript=TypeScriptTarget.TS50,
                rust=RustTarget.RUST2021,
            )
        self.assertEqual(
            str(ctx.exception),
            "TargetProfile must inhabit exactly one variant, got python, typescript, rust",
        )

    def test_target_profile_single_variant_constructions(self) -> None:
        p_py = TargetProfile(python=PythonTarget.PY311)
        p_ts = TargetProfile(typescript=TypeScriptTarget.TS50)
        p_rust = TargetProfile(rust=RustTarget.RUST2021)
        self.assertEqual(profile_id(p_py), "python-3.11")
        self.assertEqual(profile_id(p_ts), "typescript-5.0")
        self.assertEqual(profile_id(p_rust), "rust-2021")

    def test_rust_target_asymmetry(self) -> None:
        # Tell 10: RUST2021 minimum_version == "1.56", edition == "2021"
        self.assertEqual(RustTarget.RUST2021.minimum_version, "1.56")
        self.assertEqual(RustTarget.RUST2021.edition, "2021")
        self.assertFalse(RustTarget.RUST2021.reserves_gen)

        # RUST2024 minimum_version == "1.85", edition == "2024"
        self.assertEqual(RustTarget.RUST2024.minimum_version, "1.85")
        self.assertEqual(RustTarget.RUST2024.edition, "2024")
        self.assertTrue(RustTarget.RUST2024.reserves_gen)

    def test_python_target_properties(self) -> None:
        self.assertEqual(PythonTarget.PY311.minimum_version, "3.11")
        self.assertFalse(PythonTarget.PY311.uses_pep695_type_aliases)
        self.assertTrue(PythonTarget.PY312.uses_pep695_type_aliases)
        self.assertTrue(PythonTarget.PY313.uses_pep695_type_aliases)
        self.assertTrue(PythonTarget.PY314.uses_pep695_type_aliases)

    def test_typescript_target_properties(self) -> None:
        self.assertEqual(TypeScriptTarget.TS50.minimum_version, "5.0")

    def test_known_target_ids_declaration_order(self) -> None:
        expected = (
            "python-3.11",
            "python-3.12",
            "python-3.13",
            "python-3.14",
            "typescript-5.0",
            "rust-2021",
            "rust-2024",
        )
        self.assertEqual(KNOWN_TARGET_IDS, expected)

    def test_profile_id_and_lang(self) -> None:
        py_prof = TargetProfile(python=PythonTarget.PY311)
        ts_prof = TargetProfile(typescript=TypeScriptTarget.TS50)
        rust_prof = TargetProfile(rust=RustTarget.RUST2021)

        self.assertEqual(profile_id(py_prof), "python-3.11")
        self.assertEqual(profile_lang(py_prof), Lang.PY)

        self.assertEqual(profile_id(ts_prof), "typescript-5.0")
        self.assertEqual(profile_lang(ts_prof), Lang.TS)

        self.assertEqual(profile_id(rust_prof), "rust-2021")
        self.assertEqual(profile_lang(rust_prof), Lang.RUST)

    def test_profile_from_id_valid(self) -> None:
        for pid in KNOWN_TARGET_IDS:
            res = profile_from_id(pid)
            self.assertIsInstance(res, TargetProfile)

    def test_profile_from_id_unknown(self) -> None:
        res = profile_from_id("python-3.10")
        self.assertIsInstance(res, UnknownTargetProfile)

    def test_default_profile_for(self) -> None:
        # Tell 9: default for Lang.PY is python-3.11
        py_def = default_profile_for(Lang.PY)
        self.assertEqual(profile_id(py_def), "python-3.11")
        self.assertEqual(py_def.python, PythonTarget.PY311)

        ts_def = default_profile_for(Lang.TS)
        self.assertEqual(profile_id(ts_def), "typescript-5.0")

        rust_def = default_profile_for(Lang.RUST)
        self.assertEqual(profile_id(rust_def), "rust-2021")

    def test_profile_requirements_python_none_fields(self) -> None:
        # Tell 11: python has None in module_system, module_resolution, strict
        prof = TargetProfile(python=PythonTarget.PY311)
        reqs = profile_requirements(prof)
        self.assertIsNone(reqs.module_system)
        self.assertIsNone(reqs.module_resolution)
        self.assertIsNone(reqs.strict)
        self.assertEqual(reqs.compiler, "python")
        self.assertEqual(reqs.transport, "generated-h2c-and-tls-alpn-h2")
        self.assertEqual(reqs.runtime_dependencies, ("pydantic>=2",))

    def test_profile_requirements_typescript(self) -> None:
        # Tell 11: typescript has non-None module_system, module_resolution, strict
        prof = TargetProfile(typescript=TypeScriptTarget.TS50)
        reqs = profile_requirements(prof)
        self.assertEqual(reqs.module_system, "ESNext")
        self.assertEqual(reqs.module_resolution, "Bundler")
        self.assertTrue(reqs.strict)
        self.assertEqual(reqs.compiler, "typescript")
        self.assertEqual(reqs.language_standard, "ES2022")
        self.assertEqual(reqs.transport, "fetch-or-axios")
        self.assertEqual(reqs.runtime_dependencies, ())

    def test_profile_requirements_rust_none_fields(self) -> None:
        # Tell 11: rust has None in module_system, module_resolution, strict
        prof = TargetProfile(rust=RustTarget.RUST2024)
        reqs = profile_requirements(prof)
        self.assertIsNone(reqs.module_system)
        self.assertIsNone(reqs.module_resolution)
        self.assertIsNone(reqs.strict)
        self.assertEqual(reqs.compiler, "rustc")
        self.assertEqual(reqs.minimum_version, "1.85")
        self.assertEqual(reqs.language_standard, "2024")
        self.assertEqual(reqs.transport, "reqwest-blocking")
        self.assertEqual(
            reqs.runtime_dependencies, ("reqwest", "serde", "serde_json")
        )

    def test_policy_from_mapping_valid(self) -> None:
        raw = {
            "typescript": "typescript-5.0",
            "python": "python-3.12",
            "rust": "rust-2024",
        }
        res = policy_from_mapping(raw)
        self.assertIsInstance(res, TargetPolicy)
        assert isinstance(res, TargetPolicy)
        self.assertEqual(profile_id(res.python), "python-3.12")
        self.assertEqual(profile_id(res.typescript), "typescript-5.0")
        self.assertEqual(profile_id(res.rust), "rust-2024")

    def test_policy_from_mapping_missing_keys(self) -> None:
        # Tell 12: requires ALL THREE keys
        res = policy_from_mapping({})
        self.assertIsInstance(res, MissingPolicyKey)
        assert isinstance(res, MissingPolicyKey)
        self.assertEqual(res.key, "typescript")

    def test_policy_from_mapping_language_mismatch(self) -> None:
        # Tell 13: PolicyLanguageMismatch.key carries "targets." prefix
        raw = {
            "typescript": "typescript-5.0",
            "python": "rust-2021",
            "rust": "rust-2021",
        }
        res = policy_from_mapping(raw)
        self.assertIsInstance(res, PolicyLanguageMismatch)
        assert isinstance(res, PolicyLanguageMismatch)
        self.assertEqual(res.key, "targets.python")
        self.assertEqual(res.expected, Lang.PY)
        self.assertEqual(res.got, "rust-2021")

    def test_policy_from_mapping_unknown_profile(self) -> None:
        raw = {
            "typescript": "typescript-5.0",
            "python": "python-3.99",
            "rust": "rust-2021",
        }
        res = policy_from_mapping(raw)
        self.assertIsInstance(res, UnknownTargetProfile)
        assert isinstance(res, UnknownTargetProfile)
        self.assertEqual(res.value, "python-3.99")

    def test_policy_keys_constant(self) -> None:
        self.assertEqual(POLICY_KEYS, ("typescript", "python", "rust"))

    def test_target_policy_resolve(self) -> None:
        policy = TargetPolicy(
            typescript=TargetProfile(typescript=TypeScriptTarget.TS50),
            python=TargetProfile(python=PythonTarget.PY311),
            rust=TargetProfile(rust=RustTarget.RUST2021),
        )
        # Default resolution
        res_py = policy.resolve(Lang.PY, None)
        self.assertIsInstance(res_py, TargetProfile)
        assert isinstance(res_py, TargetProfile)
        self.assertEqual(profile_id(res_py), "python-3.11")

        # Explicit valid resolution
        res_py_exp = policy.resolve(Lang.PY, "python-3.14")
        self.assertIsInstance(res_py_exp, TargetProfile)

        # Mismatch resolution
        res_mismatch = policy.resolve(Lang.PY, "rust-2021")
        self.assertIsInstance(res_mismatch, TargetLanguageMismatch)


if __name__ == "__main__":
    unittest.main()
