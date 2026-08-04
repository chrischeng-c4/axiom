from __future__ import annotations

import ast
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

class TestLayerBoundaries(unittest.TestCase):
    def setUp(self) -> None:
        self.src_dir = Path(__file__).resolve().parents[2] / "src" / "storage_durable"

    def _get_imports(self, file_path: Path) -> tuple[list[str], list[str]]:
        tree = ast.parse(file_path.read_text(), filename=str(file_path))
        imports: list[str] = []
        from_imports: list[str] = []
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    imports.append(alias.name)
            elif isinstance(node, ast.ImportFrom):
                if node.module:
                    from_imports.append(node.module)
        return imports, from_imports

    def test_domain_layer_imports(self) -> None:
        domain_dir = self.src_dir / "domain"
        for py_file in domain_dir.glob("*.py"):
            if py_file.name == "__init__.py":
                continue
            imports, from_imports = self._get_imports(py_file)
            for imp in imports + from_imports:
                self.assertNotIn("application", imp, f"Domain file {py_file.name} imports application: {imp}")
                self.assertNotIn("infrastructure", imp, f"Domain file {py_file.name} imports infrastructure: {imp}")
                self.assertNotEqual(imp, "os", f"Domain file {py_file.name} imports os")
                self.assertNotEqual(imp, "pathlib", f"Domain file {py_file.name} imports pathlib")

    def test_application_layer_imports(self) -> None:
        app_dir = self.src_dir / "application"
        for py_file in app_dir.glob("*.py"):
            if py_file.name == "__init__.py":
                continue
            imports, from_imports = self._get_imports(py_file)
            for imp in imports + from_imports:
                self.assertNotIn("memory_filesystem", imp, f"Application file {py_file.name} imports memory_filesystem: {imp}")

    def test_ports_imports(self) -> None:
        ports_file = self.src_dir / "infrastructure" / "ports.py"
        imports, from_imports = self._get_imports(ports_file)
        for imp in imports + from_imports:
            self.assertNotIn("memory_filesystem", imp, f"ports.py imports memory_filesystem: {imp}")

    def test_memory_filesystem_imports(self) -> None:
        mem_fs_file = self.src_dir / "infrastructure" / "memory_filesystem.py"
        imports, from_imports = self._get_imports(mem_fs_file)
        for imp in imports + from_imports:
            self.assertNotIn("application", imp, f"memory_filesystem.py imports application: {imp}")

if __name__ == "__main__":
    unittest.main()
