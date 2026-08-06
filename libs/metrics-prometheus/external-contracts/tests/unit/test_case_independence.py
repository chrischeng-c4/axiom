from __future__ import annotations

import ast
from pathlib import Path
import unittest

CASE_COUNTS = {
    "lock-free-accumulation-behavior": 14,
    "lock-free-accumulation-security": 14,
    "exposition-encoding-behavior": 14,
    "exposition-encoding-security": 14,
    "label-value-containment-behavior": 14,
    "label-value-containment-security": 14,
}


class TestCaseIndependence(unittest.TestCase):
    def setUp(self) -> None:
        self.src_dir = Path(__file__).resolve().parents[2] / "src"

    def test_case_file_inventory(self) -> None:
        stems = {p.stem for p in self.src_dir.glob("*.py")} - {"runner"}
        self.assertEqual(stems, set(CASE_COUNTS.keys()))

    def test_case_file_ast_structure(self) -> None:
        for stem, expected_count in CASE_COUNTS.items():
            case_path = self.src_dir / f"{stem}.py"
            self.assertTrue(case_path.exists(), f"Missing file: {case_path}")

            with open(case_path, "r", encoding="utf-8") as f:
                tree = ast.parse(f.read(), filename=str(case_path))

            minimum_checks = None
            matrix_tuples = []
            verify_func_node = None
            imports = set()

            for node in ast.iter_child_nodes(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        imports.add(alias.name)
                elif isinstance(node, ast.ImportFrom):
                    if node.module:
                        imports.add(node.module)
                elif isinstance(node, ast.Assign):
                    for target in node.targets:
                        if isinstance(target, ast.Name):
                            if target.id == "MINIMUM_CHECKS" and isinstance(node.value, ast.Constant):
                                minimum_checks = node.value.value
                            elif target.id.endswith("_MATRIX") and isinstance(node.value, ast.Tuple):
                                matrix_tuples.append(node.value.elts)
                elif isinstance(node, ast.FunctionDef):
                    expected_func_name = f"verify_{stem.replace('-', '_')}"
                    if node.name == expected_func_name:
                        verify_func_node = node

            # Check no unittest or pytest imports
            self.assertNotIn("unittest", imports, f"{stem} imports unittest")
            self.assertNotIn("pytest", imports, f"{stem} imports pytest")

            # Check MINIMUM_CHECKS constant
            self.assertEqual(minimum_checks, expected_count, f"{stem} MINIMUM_CHECKS != {expected_count}")

            # Check matrix length
            self.assertEqual(len(matrix_tuples), 1, f"{stem} must have exactly 1 MATRIX assignment")
            self.assertEqual(len(matrix_tuples[0]), expected_count, f"{stem} matrix tuple length != {expected_count}")

            # Check verify function exists
            self.assertIsNotNone(verify_func_node, f"{stem} missing verify function")

            # Count checks.append calls and ensure none inside loops/comprehensions
            appends = []
            for child in ast.walk(verify_func_node): # type: ignore
                if isinstance(child, ast.Call):
                    func = child.func
                    if isinstance(func, ast.Attribute) and func.attr == "append":
                        if isinstance(func.value, ast.Name) and func.value.id == "checks":
                            appends.append(child)

            self.assertEqual(len(appends), expected_count, f"{stem} verify function appends != {expected_count}")

            # Verify no appends inside For, While, ListComp, DictComp, SetComp, GeneratorExp
            for child_node in ast.walk(verify_func_node): # type: ignore
                if isinstance(child_node, (ast.For, ast.While, ast.ListComp, ast.DictComp, ast.SetComp, ast.GeneratorExp)):
                    for sub in ast.walk(child_node):
                        if isinstance(sub, ast.Call):
                            f_sub = sub.func
                            if isinstance(f_sub, ast.Attribute) and f_sub.attr == "append":
                                if isinstance(f_sub.value, ast.Name) and f_sub.value.id == "checks":
                                    self.fail(f"{stem} contains checks.append inside loop/comprehension")


if __name__ == "__main__":
    unittest.main()
