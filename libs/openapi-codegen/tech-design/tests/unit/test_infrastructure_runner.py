from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.domain.lang import Lang
from openapi_codegen.domain.output_path import OutputPathEscape
from openapi_codegen.domain.target import default_profile_for
from openapi_codegen.infrastructure.manifest import (
    MANIFEST_FILE,
    manifest_of,
    serialize_manifest,
)
from openapi_codegen.infrastructure.options import (
    GenOptions,
    GeneratedFile,
    GeneratedOutput,
    HttpClient,
    for_target,
    legacy,
)
from openapi_codegen.infrastructure.runner import RunResult, run, write_plan


class FakeFileSystem:
    def __init__(self, files: dict[str, str] | None = None) -> None:
        self.read_store: dict[str, str] = files if files is not None else {}
        self.write_log: list[tuple[str, str]] = []

    def read_text(self, path: str) -> str | None:
        return self.read_store.get(path)

    def write_text(self, path: str, contents: str) -> None:
        self.write_log.append((path, contents))


class TestInfrastructureRunner(unittest.TestCase):
    def make_opts(self) -> GenOptions:
        return GenOptions(
            lang=Lang.TS,
            target=None,
            spec_path="/spec.json",
            out_dir="/out",
            client_name="Client",
            http_client=HttpClient.FETCH,
            emit_types=True,
            emit_client=True,
            emit_hooks=True,
        )

    def test_write_plan_legacy_output(self) -> None:
        f1 = GeneratedFile("types.ts", "// t")
        f2 = GeneratedFile("index.ts", "// i")
        out = legacy((f1, f2))
        plan = write_plan(out, "/out")
        self.assertIsInstance(plan, tuple)
        assert isinstance(plan, tuple)
        self.assertEqual(
            plan,
            (("/out/types.ts", "// t"), ("/out/index.ts", "// i")),
        )

    def test_write_plan_targeted_output_adds_manifest_last(self) -> None:
        prof = default_profile_for(Lang.TS)
        f1 = GeneratedFile("types.ts", "// t")
        out = for_target([f1], prof)
        m = manifest_of(out)
        assert m is not None
        plan = write_plan(out, "/out")
        self.assertIsInstance(plan, tuple)
        assert isinstance(plan, tuple)
        self.assertEqual(
            plan,
            (
                ("/out/types.ts", "// t"),
                (f"/out/{MANIFEST_FILE}", serialize_manifest(m)),
            ),
        )

    def test_write_plan_path_escape_aborts_entire_plan(self) -> None:
        f1 = GeneratedFile("legal.ts", "// legal")
        f2 = GeneratedFile("../escape.ts", "// bad")
        out = legacy((f1, f2))
        plan = write_plan(out, "/out")
        self.assertIsInstance(plan, OutputPathEscape)

    def test_run_exit_code_2_spec_unreadable(self) -> None:
        opts = self.make_opts()
        fs = FakeFileSystem()
        res = run(opts, fs, lambda s, o: legacy(()))
        self.assertEqual(res.exit_code, 2)
        self.assertEqual(res.stdout, ())
        self.assertEqual(
            res.stderr, ("openapi-codegen: cannot read /spec.json",)
        )
        self.assertEqual(fs.write_log, [])

    def test_run_exit_code_1_generation_error(self) -> None:
        opts = self.make_opts()
        fs = FakeFileSystem({"/spec.json": "{}"})

        def fake_gen(s: str, o: GenOptions) -> str:
            return "spec syntax error"

        res = run(opts, fs, fake_gen)
        self.assertEqual(res.exit_code, 1)
        self.assertEqual(res.stdout, ())
        self.assertEqual(res.stderr, ("openapi-codegen: spec syntax error",))
        self.assertEqual(fs.write_log, [])

    def test_run_exit_code_1_path_escape(self) -> None:
        opts = self.make_opts()
        fs = FakeFileSystem({"/spec.json": "{}"})
        bad_file = GeneratedFile("../escape.ts", "// bad")

        def fake_gen(s: str, o: GenOptions) -> GeneratedOutput:
            return legacy((bad_file,))

        res = run(opts, fs, fake_gen)
        self.assertEqual(res.exit_code, 1)
        self.assertEqual(res.stdout, ())
        self.assertTrue(res.stderr[0].startswith("openapi-codegen: cannot write generated output to /out:"))
        self.assertEqual(fs.write_log, [])

    def test_run_exit_code_0_success_legacy(self) -> None:
        opts = self.make_opts()
        fs = FakeFileSystem({"/spec.json": "{}"})
        f1 = GeneratedFile("types.ts", "// t")

        def fake_gen(s: str, o: GenOptions) -> GeneratedOutput:
            return legacy((f1,))

        res = run(opts, fs, fake_gen)
        self.assertEqual(res.exit_code, 0)
        self.assertEqual(res.stderr, ())
        self.assertEqual(res.stdout, ("generated /out/types.ts",))
        self.assertEqual(fs.write_log, [("/out/types.ts", "// t")])

    def test_run_exit_code_0_success_targeted(self) -> None:
        prof = default_profile_for(Lang.TS)
        opts = GenOptions(
            lang=Lang.TS,
            target=prof,
            spec_path="/spec.json",
            out_dir="/out",
            client_name="Client",
            http_client=HttpClient.FETCH,
            emit_types=True,
            emit_client=True,
            emit_hooks=True,
        )
        fs = FakeFileSystem({"/spec.json": "{}"})
        f1 = GeneratedFile("types.ts", "// t")

        def fake_gen(s: str, o: GenOptions) -> GeneratedOutput:
            return for_target([f1], prof)

        out = for_target([f1], prof)
        m = manifest_of(out)
        assert m is not None

        res = run(opts, fs, fake_gen)
        self.assertEqual(res.exit_code, 0)
        self.assertEqual(res.stderr, ())
        self.assertEqual(
            res.stdout,
            ("generated /out/types.ts", f"generated /out/{MANIFEST_FILE}"),
        )
        self.assertEqual(
            fs.write_log,
            [
                ("/out/types.ts", "// t"),
                (f"/out/{MANIFEST_FILE}", serialize_manifest(m)),
            ],
        )

    def test_run_result_dataclass(self) -> None:
        r = RunResult(exit_code=0, stdout=("a",), stderr=())
        self.assertEqual(r.exit_code, 0)
        self.assertEqual(r.stdout, ("a",))

    def test_write_plan_preserves_file_order(self) -> None:
        f1 = GeneratedFile("a.ts", "a")
        f2 = GeneratedFile("b.ts", "b")
        out = legacy((f1, f2))
        plan = write_plan(out, "/out")
        assert isinstance(plan, tuple)
        self.assertEqual(plan[0][0], "/out/a.ts")
        self.assertEqual(plan[1][0], "/out/b.ts")

    def test_run_spec_unreadable_write_log_empty(self) -> None:
        opts = self.make_opts()
        fs = FakeFileSystem()
        run(opts, fs, lambda s, o: legacy(()))
        self.assertEqual(fs.write_log, [])

    def test_run_path_escape_non_first_position_fails_closed(self) -> None:
        opts = self.make_opts()
        fs = FakeFileSystem({"/spec.json": "{}"})
        f1 = GeneratedFile("good.ts", "// good")
        f2 = GeneratedFile("../bad.ts", "// bad")

        def fake_gen(s: str, o: GenOptions) -> GeneratedOutput:
            return legacy((f1, f2))

        res = run(opts, fs, fake_gen)
        self.assertEqual(res.exit_code, 1)
        self.assertEqual(fs.write_log, [])

    def test_run_stdout_tuple(self) -> None:
        opts = self.make_opts()
        fs = FakeFileSystem({"/spec.json": "{}"})
        res = run(opts, fs, lambda s, o: legacy(()))
        self.assertIsInstance(res.stdout, tuple)

    def test_run_stderr_tuple(self) -> None:
        opts = self.make_opts()
        fs = FakeFileSystem()
        res = run(opts, fs, lambda s, o: legacy(()))
        self.assertIsInstance(res.stderr, tuple)

    def test_fake_filesystem_protocol(self) -> None:
        fs = FakeFileSystem({"/a": "hello"})
        self.assertEqual(fs.read_text("/a"), "hello")
        self.assertIsNone(fs.read_text("/b"))
        fs.write_text("/b", "world")
        self.assertEqual(fs.write_log, [("/b", "world")])

    def test_run_multiple_files_stdout_order(self) -> None:
        opts = self.make_opts()
        fs = FakeFileSystem({"/spec.json": "{}"})
        f1 = GeneratedFile("types.ts", "t")
        f2 = GeneratedFile("client.ts", "c")

        def fake_gen(s: str, o: GenOptions) -> GeneratedOutput:
            return legacy((f1, f2))

        res = run(opts, fs, fake_gen)
        self.assertEqual(
            res.stdout, ("generated /out/types.ts", "generated /out/client.ts")
        )

    def test_run_targeted_manifest_stdout_last(self) -> None:
        prof = default_profile_for(Lang.PY)
        opts = GenOptions(
            lang=Lang.PY,
            target=prof,
            spec_path="/spec.json",
            out_dir="/out",
            client_name="Client",
            http_client=HttpClient.FETCH,
            emit_types=True,
            emit_client=True,
            emit_hooks=False,
        )
        fs = FakeFileSystem({"/spec.json": "{}"})
        f1 = GeneratedFile("models.py", "m")

        def fake_gen(s: str, o: GenOptions) -> GeneratedOutput:
            return for_target([f1], prof)

        res = run(opts, fs, fake_gen)
        self.assertEqual(
            res.stdout,
            ("generated /out/models.py", f"generated /out/{MANIFEST_FILE}"),
        )

    def test_run_passes_spec_text_to_generate(self) -> None:
        opts = self.make_opts()
        fs = FakeFileSystem({"/spec.json": "custom spec content"})
        received_spec: list[str] = []

        def fake_gen(s: str, o: GenOptions) -> GeneratedOutput:
            received_spec.append(s)
            return legacy(())

        res = run(opts, fs, fake_gen)
        self.assertEqual(res.exit_code, 0)
        self.assertEqual(received_spec, ["custom spec content"])

    def test_write_plan_absolute_path_escapes(self) -> None:
        abs_file = GeneratedFile("/etc/passwd", "// bad")
        out = legacy((abs_file,))
        plan = write_plan(out, "/out")
        self.assertIsInstance(plan, OutputPathEscape)
        assert isinstance(plan, OutputPathEscape)
        self.assertEqual(plan.reason, "absolute")


if __name__ == "__main__":
    unittest.main()
