from __future__ import annotations

import ast
import unittest
from pathlib import Path

EXPECTED_CASES = {
    "versioned-log-envelope-behavior": 15,
    "versioned-log-envelope-security": 12,
    "correlation-field-integrity-behavior": 12,
    "correlation-field-integrity-security": 14,
    "attribute-containment-behavior": 11,
    "attribute-containment-security": 15,
    "degraded-telemetry-fallback-behavior": 11,
    "degraded-telemetry-fallback-security": 10,
    "connection-lifecycle-metrics-behavior": 11,
    "connection-lifecycle-metrics-security": 11,
    "portable-process-sampling-behavior": 10,
    "portable-process-sampling-security": 14,
}

BANNED_IMPORTS = {
    "unittest",
    "pytest",
    "hypothesis",
    "nose",
    "os",
    "sys",
    "re",
    "json",
    "math",
    "random",
    "time",
    "pathlib",
    "subprocess",
    "hashlib",
}

# service-observability owns no clock and no storage: the timestamp is handed in
# by the caller and the process sample is a string the platform already produced.
# A case therefore needs nothing from outside the design package, and every value
# it observes must come from `service_observability` itself.
ALLOWED_NON_DESIGN_IMPORTS = {"__future__"}


class TestCaseIndependence(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.here = Path(__file__).resolve().parent
        cls.src_dir = cls.here.parents[1] / "src"

    def test_every_declared_case_file_exists(self) -> None:
        for case_id in EXPECTED_CASES:
            case_path = self.src_dir / f"{case_id}.py"
            self.assertTrue(case_path.is_file(), f"missing case file: {case_path}")

        stems = {p.stem for p in self.src_dir.glob("*.py")} - {"runner"}
        self.assertEqual(stems, set(EXPECTED_CASES.keys()))

    def test_each_case_has_exactly_one_verify_entrypoint(self) -> None:
        for case_id in EXPECTED_CASES:
            with self.subTest(case_id=case_id):
                case_path = self.src_dir / f"{case_id}.py"
                tree = ast.parse(case_path.read_text(encoding="utf-8"))
                top_level_funcs = [
                    node.name
                    for node in tree.body
                    if isinstance(node, ast.FunctionDef)
                    and node.name.startswith("verify_")
                ]
                expected_name = f"verify_{case_id.replace('-', '_')}"
                self.assertEqual(len(top_level_funcs), 1)
                self.assertEqual(top_level_funcs[0], expected_name)

    def test_declared_arity_matches_the_matrix_and_the_appends(self) -> None:
        for case_id, expected_count in EXPECTED_CASES.items():
            with self.subTest(case_id=case_id):
                case_path = self.src_dir / f"{case_id}.py"
                tree = ast.parse(case_path.read_text(encoding="utf-8"))

                min_checks = None
                matrix_len = None
                for stmt in tree.body:
                    if isinstance(stmt, ast.Assign):
                        for target in stmt.targets:
                            if isinstance(target, ast.Name):
                                if (
                                    target.id == "MINIMUM_CHECKS"
                                    and isinstance(stmt.value, ast.Constant)
                                    and isinstance(stmt.value.value, int)
                                ):
                                    min_checks = stmt.value.value
                                elif target.id.endswith("_MATRIX") and isinstance(
                                    stmt.value, (ast.Tuple, ast.List)
                                ):
                                    matrix_len = len(stmt.value.elts)

                appends = 0
                for node in ast.walk(tree):
                    if (
                        isinstance(node, ast.Call)
                        and isinstance(node.func, ast.Attribute)
                        and node.func.attr == "append"
                        and isinstance(node.func.value, ast.Name)
                        and node.func.value.id == "checks"
                    ):
                        appends += 1

                self.assertIsNotNone(min_checks, f"{case_id}: no MINIMUM_CHECKS")
                self.assertIsNotNone(matrix_len, f"{case_id}: no _MATRIX")
                self.assertEqual(min_checks, expected_count)
                self.assertEqual(matrix_len, expected_count)
                self.assertEqual(appends, expected_count)

    def test_no_append_is_generated_by_a_loop(self) -> None:
        for case_id in EXPECTED_CASES:
            with self.subTest(case_id=case_id):
                case_path = self.src_dir / f"{case_id}.py"
                tree = ast.parse(case_path.read_text(encoding="utf-8"))

                append_calls = []
                for node in ast.walk(tree):
                    if (
                        isinstance(node, ast.Call)
                        and isinstance(node.func, ast.Attribute)
                        and node.func.attr == "append"
                        and isinstance(node.func.value, ast.Name)
                        and node.func.value.id == "checks"
                    ):
                        append_calls.append(node)

                for node in ast.walk(tree):
                    if isinstance(
                        node,
                        (
                            ast.For,
                            ast.While,
                            ast.ListComp,
                            ast.SetComp,
                            ast.DictComp,
                            ast.GeneratorExp,
                        ),
                    ):
                        for descendant in ast.walk(node):
                            if descendant in append_calls:
                                self.fail(
                                    f"{case_id}: checks.append(...) inside a loop"
                                )

    def test_no_case_imports_another_case_or_a_forbidden_module(self) -> None:
        case_names = {cid for cid in EXPECTED_CASES} | {
            cid.replace("-", "_") for cid in EXPECTED_CASES
        }
        for case_id in EXPECTED_CASES:
            with self.subTest(case_id=case_id):
                case_path = self.src_dir / f"{case_id}.py"
                tree = ast.parse(case_path.read_text(encoding="utf-8"))

                imported = set()
                for node in ast.walk(tree):
                    if isinstance(node, ast.Import):
                        for alias in node.names:
                            imported.add(alias.name)
                    elif isinstance(node, ast.ImportFrom):
                        if node.module:
                            imported.add(node.module)

                others = case_names - {case_id, case_id.replace("-", "_")}
                for mod in imported:
                    self.assertNotIn(mod, others, f"{case_id} imports another case")
                    self.assertNotIn(
                        mod.split(".")[0],
                        BANNED_IMPORTS,
                        f"{case_id} imports forbidden module {mod}",
                    )

    def test_every_case_observes_only_the_service_observability_design(self) -> None:
        for case_id in EXPECTED_CASES:
            with self.subTest(case_id=case_id):
                case_path = self.src_dir / f"{case_id}.py"
                tree = ast.parse(case_path.read_text(encoding="utf-8"))
                for node in ast.walk(tree):
                    if isinstance(node, ast.ImportFrom) and node.module:
                        if node.module in ALLOWED_NON_DESIGN_IMPORTS:
                            continue
                        self.assertTrue(
                            node.module.startswith("service_observability."),
                            f"{case_id} imports {node.module}",
                        )


if __name__ == "__main__":
    unittest.main()
