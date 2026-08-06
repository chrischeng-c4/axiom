from __future__ import annotations

import ast
from pathlib import Path
import sys
import unittest

SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
if str(SRC_ROOT) not in sys.path:
    sys.path.insert(0, str(SRC_ROOT))


class TestLayerBoundaries(unittest.TestCase):
    def test_domain_imports_no_application_or_infrastructure(self) -> None:
        domain_dir = SRC_ROOT / "build_stamp" / "domain"
        for py_file in domain_dir.glob("*.py"):
            tree = ast.parse(py_file.read_text(encoding="utf-8"), filename=str(py_file))
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        name = alias.name
                        self.assertFalse(
                            name.startswith("build_stamp.application")
                            or name.startswith("build_stamp.infrastructure")
                            or name.startswith("application")
                            or name.startswith("infrastructure"),
                            f"Domain file {py_file.name} illegally imports {name}",
                        )
                elif isinstance(node, ast.ImportFrom):
                    mod = node.module or ""
                    level = node.level
                    if level == 2 and mod.startswith(("application", "infrastructure")):
                        self.fail(f"Domain file {py_file.name} illegally imports relative {mod}")
                    self.assertFalse(
                        mod.startswith("build_stamp.application")
                        or mod.startswith("build_stamp.infrastructure")
                        or mod.startswith("application")
                        or mod.startswith("infrastructure"),
                        f"Domain file {py_file.name} illegally imports from {mod}",
                    )

    def test_ports_imports_no_sibling_adapters(self) -> None:
        ports_file = SRC_ROOT / "build_stamp" / "infrastructure" / "ports.py"
        tree = ast.parse(ports_file.read_text(encoding="utf-8"), filename=str(ports_file))
        adapter_names = {
            "static_sha_source",
            "manual_clock",
            "static_target_source",
            "set_path_probe",
            "build_stamp.infrastructure.static_sha_source",
            "build_stamp.infrastructure.manual_clock",
            "build_stamp.infrastructure.static_target_source",
            "build_stamp.infrastructure.set_path_probe",
        }
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    self.assertNotIn(
                        alias.name,
                        adapter_names,
                        f"ports.py illegally imports sibling adapter {alias.name}",
                    )
            elif isinstance(node, ast.ImportFrom):
                mod = node.module or ""
                self.assertNotIn(
                    mod,
                    adapter_names,
                    f"ports.py illegally imports from sibling adapter {mod}",
                )

    def test_application_imports_no_concrete_infrastructure(self) -> None:
        app_dir = SRC_ROOT / "build_stamp" / "application"
        allowed_infra = {"build_stamp.infrastructure.ports", "infrastructure.ports", ".ports"}
        for py_file in app_dir.glob("*.py"):
            tree = ast.parse(py_file.read_text(encoding="utf-8"), filename=str(py_file))
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        if alias.name.startswith("build_stamp.infrastructure") or alias.name.startswith("infrastructure"):
                            self.assertIn(
                                alias.name,
                                allowed_infra,
                                f"Application file {py_file.name} illegally imports concrete infra {alias.name}",
                            )
                elif isinstance(node, ast.ImportFrom):
                    mod = node.module or ""
                    level = node.level
                    if level == 2 and mod.startswith("infrastructure") and mod != "infrastructure.ports":
                        self.fail(f"Application file {py_file.name} illegally imports concrete infra {mod}")
                    elif mod.startswith("build_stamp.infrastructure") and mod != "build_stamp.infrastructure.ports":
                        self.fail(f"Application file {py_file.name} illegally imports concrete infra {mod}")


if __name__ == "__main__":
    unittest.main()
