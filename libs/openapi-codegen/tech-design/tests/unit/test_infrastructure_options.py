from __future__ import annotations

import sys
import unittest
from dataclasses import FrozenInstanceError

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.domain.lang import Lang
from openapi_codegen.domain.target import default_profile_for
from openapi_codegen.infrastructure.options import (
    GenOptions,
    GeneratedFile,
    GeneratedOutput,
    HttpClient,
    default_http_client,
    default_output,
    for_target,
    legacy,
)


class TestInfrastructureOptions(unittest.TestCase):
    def test_http_client_enum_values(self) -> None:
        self.assertEqual(HttpClient.FETCH.value, "fetch")
        self.assertEqual(HttpClient.AXIOS.value, "axios")

    def test_default_http_client(self) -> None:
        self.assertEqual(default_http_client(), HttpClient.FETCH)

    def test_gen_options_fields(self) -> None:
        opts = GenOptions(
            lang=Lang.TS,
            target=None,
            spec_path="/spec.json",
            out_dir="/out",
            client_name="PetClient",
            http_client=HttpClient.FETCH,
            emit_types=True,
            emit_client=True,
            emit_hooks=False,
        )
        self.assertEqual(opts.lang, Lang.TS)
        self.assertIsNone(opts.target)
        self.assertEqual(opts.spec_path, "/spec.json")
        self.assertEqual(opts.out_dir, "/out")
        self.assertEqual(opts.client_name, "PetClient")
        self.assertEqual(opts.http_client, HttpClient.FETCH)
        self.assertTrue(opts.emit_types)
        self.assertTrue(opts.emit_client)
        self.assertFalse(opts.emit_hooks)

    def test_gen_options_frozen_immutability(self) -> None:
        opts = GenOptions(
            lang=Lang.TS,
            target=None,
            spec_path="s",
            out_dir="o",
            client_name="c",
            http_client=HttpClient.FETCH,
            emit_types=True,
            emit_client=True,
            emit_hooks=True,
        )
        with self.assertRaises(FrozenInstanceError):
            opts.lang = Lang.PY  # type: ignore[misc]

    def test_generated_file_fields(self) -> None:
        f = GeneratedFile(rel_path="models.py", contents="# code")
        self.assertEqual(f.rel_path, "models.py")
        self.assertEqual(f.contents, "# code")

    def test_generated_file_frozen_immutability(self) -> None:
        f = GeneratedFile(rel_path="m", contents="c")
        with self.assertRaises(FrozenInstanceError):
            f.rel_path = "x"  # type: ignore[misc]

    def test_generated_output_legacy_constructor(self) -> None:
        f = GeneratedFile(rel_path="index.ts", contents="// index")
        out = legacy((f,))
        self.assertEqual(out.files, (f,))
        self.assertIsNone(out.target)
        self.assertIsNone(out.requirements)

    def test_generated_output_for_target_constructor(self) -> None:
        prof = default_profile_for(Lang.PY)
        f = GeneratedFile(rel_path="models.py", contents="# py")
        out = for_target([f], prof)
        self.assertEqual(out.files, (f,))
        self.assertEqual(out.target, prof)
        self.assertIsNotNone(out.requirements)

    def test_generated_output_default_constructor(self) -> None:
        out = default_output()
        self.assertEqual(out.files, ())
        self.assertIsNone(out.target)
        self.assertIsNone(out.requirements)

    def test_generated_output_tuple_storage(self) -> None:
        out = legacy([])
        self.assertIsInstance(out.files, tuple)

    def test_generated_output_frozen_immutability(self) -> None:
        out = legacy(())
        with self.assertRaises(FrozenInstanceError):
            out.target = default_profile_for(Lang.TS)  # type: ignore[misc]

    def test_legacy_converts_list_to_tuple(self) -> None:
        f = GeneratedFile("a", "b")
        out = legacy([f])
        self.assertEqual(out.files, (f,))

    def test_for_target_converts_list_to_tuple(self) -> None:
        prof = default_profile_for(Lang.TS)
        f = GeneratedFile("a", "b")
        out = for_target([f], prof)
        self.assertEqual(out.files, (f,))

    def test_for_target_requirements_match_target(self) -> None:
        prof = default_profile_for(Lang.RUST)
        out = for_target((), prof)
        assert out.requirements is not None
        self.assertEqual(out.requirements.language, Lang.RUST)

    def test_gen_options_hashable(self) -> None:
        opts = GenOptions(
            lang=Lang.TS,
            target=None,
            spec_path="s",
            out_dir="o",
            client_name="c",
            http_client=HttpClient.FETCH,
            emit_types=True,
            emit_client=True,
            emit_hooks=True,
        )
        self.assertIsInstance(hash(opts), int)

    def test_generated_output_hashable(self) -> None:
        out = default_output()
        self.assertIsInstance(hash(out), int)


if __name__ == "__main__":
    unittest.main()
