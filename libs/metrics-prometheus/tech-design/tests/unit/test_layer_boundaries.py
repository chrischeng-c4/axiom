from __future__ import annotations

import ast
from pathlib import Path
import unittest


class TestLayerBoundaries(unittest.TestCase):
    def test_imports_respect_ddd_layer_boundaries(self) -> None:
        """Enforces that domain, application, and infrastructure follow strict DDD import rules."""
        package_root = Path(__file__).resolve().parents[2] / "src" / "metrics_prometheus"

        for py_path in package_root.rglob("*.py"):
            relative_parts = py_path.relative_to(package_root).parts
            if not relative_parts:
                continue

            layer = relative_parts[0]  # "domain", "application", or "infrastructure"

            with open(py_path, "r", encoding="utf-8") as f:
                tree = ast.parse(f.read(), filename=str(py_path))

            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        self._check_import_rule(layer, alias.name, py_path, node.lineno)
                elif isinstance(node, ast.ImportFrom):
                    if node.module:
                        self._check_import_rule(layer, node.module, py_path, node.lineno)

    def _check_import_rule(self, layer: str, imported_module: str, py_path: Path, lineno: int) -> None:
        if layer == "domain":
            if imported_module.startswith("metrics_prometheus.application") or imported_module.startswith("metrics_prometheus.infrastructure"):
                self.fail(f"Domain module {py_path.name}:{lineno} illegal import: {imported_module}")
        elif layer == "application":
            if imported_module == "metrics_prometheus.infrastructure.recording_cell" or imported_module.startswith("metrics_prometheus.infrastructure.recording_cell"):
                self.fail(f"Application module {py_path.name}:{lineno} illegal import: {imported_module}")
        elif layer == "infrastructure":
            if imported_module.startswith("metrics_prometheus.application"):
                self.fail(f"Infrastructure module {py_path.name}:{lineno} illegal import: {imported_module}")


if __name__ == "__main__":
    unittest.main()
