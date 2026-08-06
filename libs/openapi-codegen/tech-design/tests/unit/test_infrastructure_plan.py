from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.domain.errors import TargetLanguageMismatch
from openapi_codegen.domain.lang import Lang
from openapi_codegen.domain.target import PythonTarget, TargetProfile, default_profile_for
from openapi_codegen.application import pymap
from openapi_codegen.application.document import Item, Schema
from openapi_codegen.application.typemap import TypeMap
from openapi_codegen.infrastructure.options import GenOptions, GeneratedFile, GeneratedOutput, HttpClient
from openapi_codegen.infrastructure.plan import (
    PY_HEADER,
    RUST_HEADER,
    TS_HEADER,
    ConflictingTarget,
    barrel_contents,
    barrel_path,
    dispatch_error,
    generate_for_target,
    planned_paths,
)


class TestInfrastructurePlan(unittest.TestCase):
    def make_opts(
        self,
        lang: Lang = Lang.TS,
        emit_types: bool = True,
        emit_client: bool = True,
        emit_hooks: bool = True,
    ) -> GenOptions:
        return GenOptions(
            lang=lang,
            target=None,
            spec_path="/spec.json",
            out_dir="/out",
            client_name="Client",
            http_client=HttpClient.FETCH,
            emit_types=emit_types,
            emit_client=emit_client,
            emit_hooks=emit_hooks,
        )

    def test_planned_paths_ts_all_true(self) -> None:
        opts = self.make_opts(Lang.TS, True, True, True)
        self.assertEqual(
            planned_paths(opts),
            ("types.ts", "runtime.ts", "client.ts", "hooks.ts", "index.ts"),
        )

    def test_planned_paths_ts_all_false(self) -> None:
        opts = self.make_opts(Lang.TS, False, False, False)
        self.assertEqual(planned_paths(opts), ("index.ts",))

    def test_planned_paths_ts_types_only(self) -> None:
        opts = self.make_opts(Lang.TS, True, False, False)
        self.assertEqual(planned_paths(opts), ("types.ts", "index.ts"))

    def test_planned_paths_ts_client_only(self) -> None:
        opts = self.make_opts(Lang.TS, False, True, False)
        self.assertEqual(
            planned_paths(opts), ("runtime.ts", "client.ts", "index.ts")
        )

    def test_planned_paths_ts_hooks_only(self) -> None:
        opts = self.make_opts(Lang.TS, False, False, True)
        self.assertEqual(planned_paths(opts), ("hooks.ts", "index.ts"))

    def test_planned_paths_py_all_true(self) -> None:
        opts = self.make_opts(Lang.PY, True, True, True)
        self.assertEqual(
            planned_paths(opts),
            ("models.py", "h2c_runtime.py", "client.py", "__init__.py"),
        )

    def test_planned_paths_py_all_false(self) -> None:
        opts = self.make_opts(Lang.PY, False, False, False)
        self.assertEqual(planned_paths(opts), ("__init__.py",))

    def test_planned_paths_py_ignores_emit_hooks(self) -> None:
        opts1 = self.make_opts(Lang.PY, True, True, False)
        opts2 = self.make_opts(Lang.PY, True, True, True)
        self.assertEqual(planned_paths(opts1), planned_paths(opts2))

    def test_planned_paths_rust_all_true(self) -> None:
        opts = self.make_opts(Lang.RUST, True, True, True)
        self.assertEqual(
            planned_paths(opts), ("models.rs", "client.rs", "mod.rs")
        )

    def test_planned_paths_rust_all_false(self) -> None:
        opts = self.make_opts(Lang.RUST, False, False, False)
        self.assertEqual(planned_paths(opts), ("mod.rs",))

    def test_planned_paths_rust_ignores_hooks_and_runtime(self) -> None:
        opts = self.make_opts(Lang.RUST, False, True, True)
        self.assertEqual(planned_paths(opts), ("client.rs", "mod.rs"))

    def test_barrel_path(self) -> None:
        self.assertEqual(barrel_path(Lang.TS), "index.ts")
        self.assertEqual(barrel_path(Lang.PY), "__init__.py")
        self.assertEqual(barrel_path(Lang.RUST), "mod.rs")

    def test_barrel_contents_ts_all_true(self) -> None:
        opts = self.make_opts(Lang.TS, True, True, True)
        expected = (
            TS_HEADER
            + 'export * from "./types";\n'
            + 'export * from "./runtime";\n'
            + 'export * from "./client";\n'
            + 'export * from "./hooks";\n'
        )
        self.assertEqual(barrel_contents(opts), expected)

    def test_barrel_contents_ts_all_false(self) -> None:
        opts = self.make_opts(Lang.TS, False, False, False)
        self.assertEqual(barrel_contents(opts), TS_HEADER)

    def test_barrel_contents_ts_single_flags(self) -> None:
        opts = self.make_opts(Lang.TS, True, False, False)
        self.assertEqual(barrel_contents(opts), TS_HEADER + 'export * from "./types";\n')

    def test_barrel_contents_py_all_true(self) -> None:
        opts = self.make_opts(Lang.PY, True, True, True)
        expected = (
            PY_HEADER
            + "from .models import *  # noqa: F401,F403\n"
            + "from .client import AsyncClient, Client  # noqa: F401\n"
            + "from .h2c_runtime import AsyncH2CClient, AsyncH2CConnection, AsyncH2CStream, H2CClient, H2CConnection, H2CResponse, H2CStream  # noqa: F401\n"
        )
        self.assertEqual(barrel_contents(opts), expected)

    def test_barrel_contents_py_all_false(self) -> None:
        opts = self.make_opts(Lang.PY, False, False, False)
        self.assertEqual(barrel_contents(opts), PY_HEADER)

    def test_barrel_contents_py_types_only(self) -> None:
        opts = self.make_opts(Lang.PY, True, False, False)
        expected = PY_HEADER + "from .models import *  # noqa: F401,F403\n"
        self.assertEqual(barrel_contents(opts), expected)

    def test_barrel_contents_rust_all_true(self) -> None:
        opts = self.make_opts(Lang.RUST, True, True, True)
        expected = RUST_HEADER + "pub mod models;\n" + "pub mod client;\n"
        self.assertEqual(barrel_contents(opts), expected)

    def test_barrel_contents_rust_all_false(self) -> None:
        opts = self.make_opts(Lang.RUST, False, False, False)
        self.assertEqual(barrel_contents(opts), RUST_HEADER)

    def test_barrel_contents_rust_client_only(self) -> None:
        opts = self.make_opts(Lang.RUST, False, True, False)
        expected = RUST_HEADER + "pub mod client;\n"
        self.assertEqual(barrel_contents(opts), expected)

    def test_dispatch_error_none_when_target_none(self) -> None:
        opts = self.make_opts(Lang.TS)
        self.assertIsNone(dispatch_error(opts, None))

    def test_dispatch_error_none_when_matching_target(self) -> None:
        ts_prof = default_profile_for(Lang.TS)
        opts = GenOptions(
            lang=Lang.TS,
            target=ts_prof,
            spec_path="s",
            out_dir="o",
            client_name="c",
            http_client=HttpClient.FETCH,
            emit_types=True,
            emit_client=True,
            emit_hooks=True,
        )
        self.assertIsNone(dispatch_error(opts, ts_prof))

    def test_dispatch_error_language_mismatch(self) -> None:
        py_prof = default_profile_for(Lang.PY)
        opts = self.make_opts(Lang.TS)
        err = dispatch_error(opts, py_prof)
        self.assertIsInstance(err, TargetLanguageMismatch)

    def test_dispatch_error_conflicting_target(self) -> None:
        configured = TargetProfile(python=PythonTarget.PY312)
        argument = TargetProfile(python=PythonTarget.PY311)
        opts = GenOptions(
            lang=Lang.PY,
            target=configured,
            spec_path="s",
            out_dir="o",
            client_name="c",
            http_client=HttpClient.FETCH,
            emit_types=True,
            emit_client=True,
            emit_hooks=True,
        )
        err = dispatch_error(opts, argument)
        self.assertIsInstance(err, ConflictingTarget)
        assert isinstance(err, ConflictingTarget)
        self.assertEqual(err.argument, "python-3.11")
        self.assertEqual(err.configured, "python-3.12")
        self.assertEqual(
            err.message(),
            "explicit target argument python-3.11 conflicts with GenOptions target python-3.12",
        )

    def test_dispatch_error_none_when_explicit_target_matches(self) -> None:
        prof = TargetProfile(python=PythonTarget.PY311)
        opts = GenOptions(
            lang=Lang.PY,
            target=prof,
            spec_path="s",
            out_dir="o",
            client_name="c",
            http_client=HttpClient.FETCH,
            emit_types=True,
            emit_client=True,
            emit_hooks=True,
        )
        self.assertIsNone(dispatch_error(opts, TargetProfile(python=PythonTarget.PY311)))

    def test_dispatch_error_precedence_language_mismatch_first(self) -> None:
        ts_prof = default_profile_for(Lang.TS)
        py_prof = default_profile_for(Lang.PY)
        opts = GenOptions(
            lang=Lang.TS,
            target=ts_prof,
            spec_path="s",
            out_dir="o",
            client_name="c",
            http_client=HttpClient.FETCH,
            emit_types=True,
            emit_client=True,
            emit_hooks=True,
        )
        err = dispatch_error(opts, py_prof)
        self.assertIsInstance(err, TargetLanguageMismatch)

    def test_conflicting_target_dataclass_message(self) -> None:
        ct = ConflictingTarget(argument="python-3.11", configured="python-3.12")
        self.assertEqual(
            ct.message(),
            "explicit target argument python-3.11 conflicts with GenOptions target python-3.12",
        )

    def test_planned_paths_returns_tuple(self) -> None:
        opts = self.make_opts()
        self.assertIsInstance(planned_paths(opts), tuple)

    def test_headers_byte_exactness(self) -> None:
        self.assertTrue(TS_HEADER.endswith("\n\n"))
        self.assertTrue(PY_HEADER.endswith("\n"))
        self.assertFalse(PY_HEADER.endswith("\n\n"))
        self.assertTrue(RUST_HEADER.endswith("\n"))
        self.assertFalse(RUST_HEADER.endswith("\n\n"))

    def test_generate_for_target_success(self) -> None:
        opts = self.make_opts(Lang.PY)
        res = generate_for_target(opts, "python-3.11", lambda t: [GeneratedFile("m.py", "# m")])
        self.assertIsInstance(res, GeneratedOutput)
        assert isinstance(res, GeneratedOutput)
        self.assertIsNotNone(res.target)

    def test_generate_for_target_profileless_callback_preserves_legacy_python(self) -> None:
        opts = self.make_opts(Lang.PY)
        seen: list[TargetProfile | None] = []

        def callback(target: TargetProfile | None) -> list[GeneratedFile]:
            seen.append(target)
            python_target = target.python if target is not None else None
            rendered = pymap.type_expr(Item(Schema(ty=("string",), nullable=True)), TypeMap(()), python_target)
            return [GeneratedFile("models.py", rendered)]

        res = generate_for_target(opts, None, callback)
        self.assertIsInstance(res, GeneratedOutput)
        assert isinstance(res, GeneratedOutput)
        self.assertEqual(seen, [None])
        self.assertIsNone(res.target)
        self.assertIsNone(res.requirements)
        self.assertEqual(res.files[0].contents, "Optional[str]")

    def test_generate_for_target_mismatch_precedes_configured_conflict(self) -> None:
        opts = GenOptions(
            Lang.PY,
            TargetProfile(python=PythonTarget.PY312),
            "/spec.json", "/out", "Client", HttpClient.FETCH,
            True, True, True,
        )
        called = False

        def callback(_: object) -> list[GeneratedFile]:
            nonlocal called
            called = True
            return []

        res = generate_for_target(opts, "rust-2021", callback)
        self.assertIsInstance(res, TargetLanguageMismatch)
        self.assertFalse(called)


if __name__ == "__main__":
    unittest.main()
