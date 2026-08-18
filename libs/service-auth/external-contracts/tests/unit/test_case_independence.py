from __future__ import annotations

import ast
import unittest
from pathlib import Path

EXPECTED_CASES = {
    "static-role-map-authorization-behavior": 14,
    "static-role-map-authorization-security": 15,
    "delegated-kubernetes-authorization-behavior": 15,
    "delegated-kubernetes-authorization-security": 17,
    "credential-reload-audit-behavior": 15,
    "credential-reload-audit-security": 16,
}


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
                code = case_path.read_text(encoding="utf-8")
                tree = ast.parse(code, filename=str(case_path))

                top_level_funcs = [
                    node.name
                    for node in tree.body
                    if isinstance(node, ast.FunctionDef)
                    and node.name.startswith("verify_")
                ]
                expected_name = f"verify_{case_id.replace('-', '_')}"
                self.assertEqual(
                    len(top_level_funcs),
                    1,
                    f"case {case_id} must have exactly one top-level verify_* function, got {top_level_funcs}",
                )
                self.assertEqual(
                    top_level_funcs[0],
                    expected_name,
                    f"case {case_id} verify function name mismatch: expected {expected_name}, got {top_level_funcs[0]}",
                )

    def test_declared_arity_matches_the_matrix_and_the_appends(self) -> None:
        for case_id, expected_count in EXPECTED_CASES.items():
            with self.subTest(case_id=case_id):
                case_path = self.src_dir / f"{case_id}.py"
                code = case_path.read_text(encoding="utf-8")
                tree = ast.parse(code, filename=str(case_path))

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
                    if isinstance(node, ast.Call):
                        if (
                            isinstance(node.func, ast.Attribute)
                            and node.func.attr == "append"
                        ):
                            if (
                                isinstance(node.func.value, ast.Name)
                                and node.func.value.id == "checks"
                            ):
                                appends += 1

                self.assertIsNotNone(
                    min_checks, f"case {case_id} missing MINIMUM_CHECKS constant"
                )
                self.assertIsNotNone(
                    matrix_len, f"case {case_id} missing _MATRIX constant"
                )
                self.assertEqual(
                    min_checks,
                    expected_count,
                    f"case {case_id} MINIMUM_CHECKS mismatch",
                )
                self.assertEqual(
                    matrix_len,
                    expected_count,
                    f"case {case_id} _MATRIX length mismatch",
                )
                self.assertEqual(
                    appends,
                    expected_count,
                    f"case {case_id} checks.append count mismatch",
                )

    def test_no_append_is_generated_by_a_loop(self) -> None:
        for case_id in EXPECTED_CASES:
            with self.subTest(case_id=case_id):
                case_path = self.src_dir / f"{case_id}.py"
                code = case_path.read_text(encoding="utf-8")
                tree = ast.parse(code, filename=str(case_path))

                append_calls = []
                for node in ast.walk(tree):
                    if isinstance(node, ast.Call):
                        if (
                            isinstance(node.func, ast.Attribute)
                            and node.func.attr == "append"
                        ):
                            if (
                                isinstance(node.func.value, ast.Name)
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
                                    f"case {case_id} contains checks.append(...) inside a loop or comprehension"
                                )

    def test_no_case_imports_another_case_or_a_test_framework(self) -> None:
        case_names = {cid for cid in EXPECTED_CASES} | {
            cid.replace("-", "_") for cid in EXPECTED_CASES
        }
        for case_id in EXPECTED_CASES:
            with self.subTest(case_id=case_id):
                case_path = self.src_dir / f"{case_id}.py"
                code = case_path.read_text(encoding="utf-8")
                tree = ast.parse(code, filename=str(case_path))

                imported_modules = set()
                for node in ast.walk(tree):
                    if isinstance(node, ast.Import):
                        for alias in node.names:
                            imported_modules.add(alias.name)
                    elif isinstance(node, ast.ImportFrom):
                        if node.module:
                            imported_modules.add(node.module)

                other_case_names = case_names - {
                    case_id,
                    case_id.replace("-", "_"),
                }

                for mod in imported_modules:
                    self.assertNotIn(
                        mod,
                        other_case_names,
                        f"case {case_id} illegally imports another case module {mod}",
                    )
                    self.assertNotIn(
                        mod,
                        {"unittest", "pytest", "hypothesis", "nose"},
                        f"case {case_id} illegally imports test framework {mod}",
                    )
                    if mod.startswith("service_auth.infrastructure."):
                        self.assertEqual(
                            mod,
                            "service_auth.infrastructure.ports",
                            f"case {case_id} illegally imports non-ports infrastructure module {mod}",
                        )


if __name__ == "__main__":
    unittest.main()
