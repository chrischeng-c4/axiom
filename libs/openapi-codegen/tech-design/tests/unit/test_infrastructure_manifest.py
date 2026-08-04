from __future__ import annotations

import sys
import unittest
from dataclasses import FrozenInstanceError

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.domain.lang import Lang
from openapi_codegen.domain.target import default_profile_for
from openapi_codegen.infrastructure.manifest import (
    MANIFEST_FILE,
    GenerationManifest,
    manifest_fields,
    manifest_of,
)
from openapi_codegen.infrastructure.options import (
    GeneratedOutput,
    for_target,
    legacy,
)


class TestInfrastructureManifest(unittest.TestCase):
    def test_manifest_file_constant(self) -> None:
        self.assertEqual(MANIFEST_FILE, ".openapi-codegen.json")

    def test_manifest_of_legacy_output_returns_none(self) -> None:
        out = legacy(())
        self.assertIsNone(manifest_of(out))

    def test_manifest_of_targeted_ts(self) -> None:
        prof = default_profile_for(Lang.TS)
        out = for_target((), prof)
        m = manifest_of(out)
        self.assertIsNotNone(m)
        assert m is not None
        self.assertEqual(m.language, "typescript")
        self.assertEqual(m.schema_version, 1)
        self.assertEqual(m.generator, "openapi-codegen")

    def test_manifest_of_targeted_py(self) -> None:
        prof = default_profile_for(Lang.PY)
        out = for_target((), prof)
        m = manifest_of(out)
        self.assertIsNotNone(m)
        assert m is not None
        self.assertEqual(m.language, "python")

    def test_manifest_of_targeted_rust(self) -> None:
        prof = default_profile_for(Lang.RUST)
        out = for_target((), prof)
        m = manifest_of(out)
        self.assertIsNotNone(m)
        assert m is not None
        self.assertEqual(m.language, "rust")

    def test_manifest_of_rust_optional_fields(self) -> None:
        prof = default_profile_for(Lang.RUST)
        out = for_target((), prof)
        m = manifest_of(out)
        self.assertIsNotNone(m)
        assert m is not None
        self.assertIsNone(m.module_system)
        self.assertIsNone(m.module_resolution)
        self.assertIsNone(m.strict)
        self.assertEqual(m.transport, "reqwest-blocking")

    def test_manifest_of_runtime_dependencies_tuple(self) -> None:
        prof = default_profile_for(Lang.TS)
        out = for_target((), prof)
        m = manifest_of(out)
        self.assertIsNotNone(m)
        assert m is not None
        self.assertIsInstance(m.runtime_dependencies, tuple)

    def test_manifest_fields_returns_12_tuples(self) -> None:
        prof = default_profile_for(Lang.TS)
        out = for_target((), prof)
        m = manifest_of(out)
        assert m is not None
        fields = manifest_fields(m)
        self.assertEqual(len(fields), 12)

    def test_manifest_fields_declaration_order(self) -> None:
        prof = default_profile_for(Lang.TS)
        out = for_target((), prof)
        m = manifest_of(out)
        assert m is not None
        fields = manifest_fields(m)
        keys = tuple(k for k, _ in fields)
        expected_keys = (
            "schema_version",
            "generator",
            "compiler",
            "target",
            "language",
            "minimum_version",
            "language_standard",
            "module_system",
            "module_resolution",
            "strict",
            "transport",
            "runtime_dependencies",
        )
        self.assertEqual(keys, expected_keys)

    def test_manifest_fields_preserves_none(self) -> None:
        prof = default_profile_for(Lang.RUST)
        out = for_target((), prof)
        m = manifest_of(out)
        assert m is not None
        fields = dict(manifest_fields(m))
        self.assertIn("module_system", fields)
        self.assertIsNone(fields["module_system"])

    def test_generation_manifest_frozen_immutability(self) -> None:
        prof = default_profile_for(Lang.TS)
        out = for_target((), prof)
        m = manifest_of(out)
        assert m is not None
        with self.assertRaises(FrozenInstanceError):
            m.generator = "other"  # type: ignore[misc]

    def test_manifest_of_reads_requirements_not_target(self) -> None:
        prof = default_profile_for(Lang.TS)
        # Create output with target set but requirements=None
        out = GeneratedOutput(files=(), target=prof, requirements=None)
        self.assertIsNone(manifest_of(out))

    def test_manifest_fields_type(self) -> None:
        prof = default_profile_for(Lang.TS)
        out = for_target((), prof)
        m = manifest_of(out)
        assert m is not None
        self.assertIsInstance(manifest_fields(m), tuple)

    def test_manifest_fields_value_extraction(self) -> None:
        prof = default_profile_for(Lang.TS)
        out = for_target((), prof)
        m = manifest_of(out)
        assert m is not None
        fields = dict(manifest_fields(m))
        self.assertEqual(fields["schema_version"], 1)
        self.assertEqual(fields["generator"], "openapi-codegen")
        self.assertEqual(fields["language"], "typescript")


if __name__ == "__main__":
    unittest.main()
