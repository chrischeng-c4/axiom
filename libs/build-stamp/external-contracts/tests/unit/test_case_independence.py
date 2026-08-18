from __future__ import annotations

import ast
from pathlib import Path
import sys
import unittest

_HERE = Path(__file__).resolve().parent
_SRC_DIR = _HERE.parents[1] / "src"

EXPECTED_ARITIES = {
    "version-stamp-emission-behavior": 14,
    "version-stamp-emission-security": 13,
    "best-effort-degradation-behavior": 14,
    "best-effort-degradation-security": 12,
    "directive-channel-integrity-behavior": 13,
    "directive-channel-integrity-security": 14,
}


class TestCaseIndependence(unittest.TestCase):
    def test_case_file_set_bijection(self) -> None:
        case_files = [f for f in _SRC_DIR.glob("*.py") if f.name != "runner.py"]
        stems = {f.stem for f in case_files}
        self.assertEqual(stems, set(EXPECTED_ARITIES.keys()))

    def test_single_verify_function_and_arities(self) -> None:
        forbidden_test_frameworks = {"unittest", "pytest", "hypothesis", "nose"}
        for case_name, expected_count in EXPECTED_ARITIES.items():
            py_file = _SRC_DIR / f"{case_name}.py"
            self.assertTrue(py_file.is_file(), f"Missing case file {py_file.name}")
            source = py_file.read_text(encoding="utf-8")
            tree = ast.parse(source, filename=str(py_file))

            # 1. Top level verify function
            expected_func_name = f"verify_{case_name.replace('-', '_')}"
            verify_funcs = [
                node.name
                for node in tree.body
                if isinstance(node, ast.FunctionDef) and node.name.startswith("verify_")
            ]
            self.assertEqual(
                verify_funcs,
                [expected_func_name],
                f"{py_file.name} must have exactly one top-level {expected_func_name}",
            )

            # 2. Check MINIMUM_CHECKS and _MATRIX length
            minimum_checks_val: int | None = None
            matrix_len: int | None = None

            for stmt in tree.body:
                if isinstance(stmt, ast.Assign):
                    for target in stmt.targets:
                        if isinstance(target, ast.Name):
                            if target.id == "MINIMUM_CHECKS" and isinstance(stmt.value, ast.Constant):
                                minimum_checks_val = stmt.value.value
                            elif target.id.endswith("_MATRIX") and isinstance(stmt.value, ast.Tuple):
                                matrix_len = len(stmt.value.elts)

            self.assertEqual(minimum_checks_val, expected_count, f"{py_file.name} MINIMUM_CHECKS mismatch")
            self.assertEqual(matrix_len, expected_count, f"{py_file.name} _MATRIX length mismatch")

            # 3. Literal checks.append calls & no loop wrapping
            append_count = 0
            for node in ast.walk(tree):
                if isinstance(node, (ast.For, ast.While, ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)):
                    for child in ast.walk(node):
                        if isinstance(child, ast.Call) and isinstance(child.func, ast.Attribute) and child.func.attr == "append":
                            if isinstance(child.func.value, ast.Name) and child.func.value.id == "checks":
                                self.fail(f"Loop or comprehension in {py_file.name} contains checks.append")
                elif isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == "append":
                    if isinstance(node.func.value, ast.Name) and node.func.value.id == "checks":
                        append_count += 1

            self.assertEqual(append_count, expected_count, f"{py_file.name} checks.append count mismatch")

            # 4. Check forbidden imports
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        self.assertNotIn(alias.name, forbidden_test_frameworks, f"Forbidden import {alias.name} in {py_file.name}")
                        self.assertFalse(
                            alias.name.startswith("build_stamp.infrastructure") and alias.name != "build_stamp.infrastructure.ports",
                            f"Illegal infra import {alias.name} in {py_file.name}",
                        )
                elif isinstance(node, ast.ImportFrom):
                    mod = node.module or ""
                    self.assertNotIn(mod, forbidden_test_frameworks, f"Forbidden import {mod} in {py_file.name}")
                    self.assertFalse(
                        mod.startswith("build_stamp.infrastructure") and mod != "build_stamp.infrastructure.ports",
                        f"Illegal infra import from {mod} in {py_file.name}",
                    )
                    # Check no case imports another case
                    mod_stem = mod.split(".")[-1].replace("_", "-")
                    self.assertNotIn(mod_stem, EXPECTED_ARITIES, f"Case file {py_file.name} imports sibling case {mod}")


if __name__ == "__main__":
    unittest.main()
