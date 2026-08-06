from __future__ import annotations

import ast
from pathlib import Path
import unittest

CASE_COUNTS = {
    "crash-safe-replacement-behavior": 14,
    "crash-safe-replacement-security": 13,
    "torn-tail-recovery-behavior": 14,
    "torn-tail-recovery-security": 14,
    "sequence-ordered-snapshots-behavior": 13,
    "sequence-ordered-snapshots-security": 14,
}

class TestCaseIndependence(unittest.TestCase):
    def setUp(self) -> None:
        self.src_dir = Path(__file__).resolve().parents[2] / "src"

    def test_all_cases_exist_and_match_declared_set(self) -> None:
        py_files = [p.stem for p in self.src_dir.glob("*.py") if p.stem != "runner"]
        self.assertEqual(set(py_files), set(CASE_COUNTS.keys()))

    def test_case_structure_and_arity(self) -> None:
        for cmd, expected_count in CASE_COUNTS.items():
            with self.subTest(command=cmd):
                case_file = self.src_dir / f"{cmd}.py"
                self.assertTrue(case_file.is_file())
                source_text = case_file.read_text()
                tree = ast.parse(source_text, filename=str(case_file))

                # Check MINIMUM_CHECKS constant
                min_checks_node = None
                matrix_node = None
                for stmt in tree.body:
                    if isinstance(stmt, ast.Assign):
                        for target in stmt.targets:
                            if isinstance(target, ast.Name):
                                if target.id == "MINIMUM_CHECKS":
                                    min_checks_node = stmt.value
                                elif target.id.endswith("_MATRIX"):
                                    matrix_node = stmt.value

                self.assertIsNotNone(min_checks_node, f"MINIMUM_CHECKS missing in {cmd}")
                self.assertIsInstance(min_checks_node, ast.Constant)
                self.assertEqual(min_checks_node.value, expected_count)

                self.assertIsNotNone(matrix_node, f"MATRIX missing in {cmd}")
                if isinstance(matrix_node, (ast.Tuple, ast.List)):
                    self.assertEqual(len(matrix_node.elts), expected_count)

                # Check verify_<name> function and checks.append calls
                func_name = f"verify_{cmd.replace('-', '_')}"
                func_def = None
                for stmt in tree.body:
                    if isinstance(stmt, ast.FunctionDef) and stmt.name == func_name:
                        func_def = stmt
                        break

                self.assertIsNotNone(func_def, f"Function {func_name} missing in {cmd}")

                # Count checks.append calls and check they are not in loops
                append_count = 0
                for node in ast.walk(func_def):
                    if isinstance(node, (ast.For, ast.While, ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)):
                        for child in ast.walk(node):
                            if isinstance(child, ast.Call) and isinstance(child.func, ast.Attribute) and child.func.attr == "append":
                                if isinstance(child.func.value, ast.Name) and child.func.value.id == "checks":
                                    self.fail(f"checks.append inside loop/comprehension in {cmd}")

                    if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == "append":
                        if isinstance(node.func.value, ast.Name) and node.func.value.id == "checks":
                            append_count += 1

                self.assertEqual(append_count, expected_count, f"checks.append count mismatch in {cmd}")

    def test_imports_in_cases(self) -> None:
        for cmd in CASE_COUNTS:
            with self.subTest(command=cmd):
                case_file = self.src_dir / f"{cmd}.py"
                tree = ast.parse(case_file.read_text(), filename=str(case_file))
                for node in ast.walk(tree):
                    if isinstance(node, ast.Import):
                        for alias in node.names:
                            self.assertNotIn("memory_filesystem", alias.name)
                            self.assertNotIn("unittest", alias.name)
                            self.assertNotIn("pytest", alias.name)
                    elif isinstance(node, ast.ImportFrom):
                        if node.module:
                            self.assertNotIn("memory_filesystem", node.module)
                            self.assertNotIn("unittest", node.module)
                            self.assertNotIn("pytest", node.module)

if __name__ == "__main__":
    unittest.main()
