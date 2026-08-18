"""Unit tests verifying architectural layer import boundaries."""

from __future__ import annotations

import ast
import sys
import unittest
from pathlib import Path

SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
if str(SRC_ROOT) not in sys.path:
    sys.path.insert(0, str(SRC_ROOT))


class TestLayerBoundaries(unittest.TestCase):
    def setUp(self) -> None:
        self.src_dir = SRC_ROOT / "service_auth"

    def _get_imports(self, file_path: Path) -> list[str]:
        tree = ast.parse(file_path.read_text(encoding="utf-8"), filename=str(file_path))
        imports: list[str] = []
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    imports.append(alias.name)
            elif isinstance(node, ast.ImportFrom):
                if node.module:
                    imports.append(node.module)
        return imports

    def test_domain_layer_imports_only_stdlib_or_domain(self) -> None:
        domain_dir = self.src_dir / "domain"
        for py_file in domain_dir.glob("*.py"):
            imports = self._get_imports(py_file)
            for imp in imports:
                self.assertFalse(
                    imp.startswith("service_auth.application")
                    or imp.startswith("service_auth.infrastructure"),
                    f"Domain module {py_file.name} violates layer boundary by importing {imp}",
                )

    def test_ports_imports_no_sibling_adapter(self) -> None:
        ports_file = self.src_dir / "infrastructure" / "ports.py"
        imports = self._get_imports(ports_file)
        adapter_modules = {
            "service_auth.infrastructure.memory_registry_source",
            "service_auth.infrastructure.memory_review_backend",
            "service_auth.infrastructure.manual_clock",
            "service_auth.infrastructure.recording_event_sink",
        }
        for imp in imports:
            self.assertNotIn(
                imp,
                adapter_modules,
                f"Ports module violates boundary by importing infrastructure adapter {imp}",
            )


if __name__ == "__main__":
    unittest.main()
