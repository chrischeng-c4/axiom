from __future__ import annotations

import ast
from pathlib import Path
import unittest

EXPECTED_CASES = {
    "material-validation-behavior": 15,
    "material-validation-security": 11,
    "mtls-config-construction-behavior": 12,
    "mtls-config-construction-security": 13,
    "rotation-and-reload-behavior": 11,
    "rotation-and-reload-security": 18,
}


class TestCaseIndependence(unittest.TestCase):

    @classmethod
    def setUpClass(cls) -> None:
        cls.here = Path(__file__).resolve().parent  # tests/unit
        cls.src_dir = cls.here.parents[1] / "src"

    def test_all_cases_conform_to_independence_contract(self) -> None:
        case_module_names = {cid.replace("-", "_") for cid in EXPECTED_CASES}
        case_file_stems = set(EXPECTED_CASES.keys())

        for case_id, expected_min in EXPECTED_CASES.items():
            with self.subTest(case_id=case_id):
                case_path = self.src_dir / f"{case_id}.py"
                self.assertTrue(case_path.is_file(), f"missing case file: {case_path}")

                code = case_path.read_text(encoding="utf-8")
                tree = ast.parse(code, filename=str(case_path))

                # 1. Exactly one top-level verify_* function
                top_level_funcs = [
                    node.name
                    for node in tree.body
                    if isinstance(node, ast.FunctionDef) and node.name.startswith("verify_")
                ]
                self.assertEqual(
                    len(top_level_funcs),
                    1,
                    f"case {case_id} should have exactly one top-level verify_* function, found {top_level_funcs}",
                )

                # 2. MINIMUM_CHECKS constant matches expected integer
                min_checks_val = None
                matrix_const_found = False

                for node in tree.body:
                    if isinstance(node, ast.Assign):
                        for target in node.targets:
                            if isinstance(target, ast.Name):
                                if target.id == "MINIMUM_CHECKS":
                                    if isinstance(node.value, ast.Constant):
                                        min_checks_val = node.value.value
                                if target.id.endswith("_MATRIX") or target.id.endswith("_EXPECTATIONS"):
                                    matrix_const_found = True

                self.assertEqual(
                    min_checks_val,
                    expected_min,
                    f"case {case_id} MINIMUM_CHECKS expected {expected_min}, got {min_checks_val}",
                )

                # 3. At least one module-level constant ending in _MATRIX or _EXPECTATIONS
                self.assertTrue(
                    matrix_const_found,
                    f"case {case_id} missing module-level constant ending in _MATRIX or _EXPECTATIONS",
                )

                # 4. No import of another case module
                other_cases_names = (case_module_names | case_file_stems) - {
                    case_id,
                    case_id.replace("-", "_"),
                }

                imported_modules = set()
                for node in ast.walk(tree):
                    if isinstance(node, ast.Import):
                        for alias in node.names:
                            imported_modules.add(alias.name)
                    elif isinstance(node, ast.ImportFrom):
                        if node.module:
                            imported_modules.add(node.module)

                forbidden_imports = imported_modules.intersection(other_cases_names)
                self.assertEqual(
                    len(forbidden_imports),
                    0,
                    f"case {case_id} illegally imports other case modules: {forbidden_imports}",
                )


if __name__ == "__main__":
    unittest.main()
