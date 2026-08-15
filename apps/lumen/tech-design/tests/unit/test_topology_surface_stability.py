"""Standing surface-stability check for lumen public types and bindings."""
from __future__ import annotations

import importlib
import inspect
import pathlib
import pkgutil
import sys
import unittest
from typing import Any

SRC_DIR = pathlib.Path(__file__).parents[2] / "src"
if str(SRC_DIR) not in sys.path:
    sys.path.insert(0, str(SRC_DIR))

import lumen

MIN_DISCOVERED_MODULES: int = 10
MIN_PUBLIC_BINDINGS: int = 20


def _discover_lumen_modules() -> list[Any]:
    modules = [lumen]
    for module_info in pkgutil.walk_packages(lumen.__path__, prefix=f"{lumen.__name__}."):
        mod = importlib.import_module(module_info.name)
        modules.append(mod)
    return modules


def _collect_lumen_public_bindings() -> tuple[list[Any], list[tuple[Any, str, Any]]]:
    modules = _discover_lumen_modules()
    bindings: list[tuple[Any, str, Any]] = []
    for mod in modules:
        for name in dir(mod):
            if name.startswith("_"):
                continue
            try:
                obj = getattr(mod, name)
            except AttributeError:
                continue
            if not (inspect.isclass(obj) or inspect.isfunction(obj)):
                continue
            decl_mod_name = getattr(obj, "__module__", None)
            if not decl_mod_name or not (decl_mod_name == "lumen" or decl_mod_name.startswith("lumen.")):
                continue
            bindings.append((mod, name, obj))
    return modules, bindings


class TestTopologySurfaceStability(unittest.TestCase):
    def test_lumen_surface_population_floor(self) -> None:
        modules, bindings = _collect_lumen_public_bindings()
        self.assertGreaterEqual(
            len(modules),
            MIN_DISCOVERED_MODULES,
            f"Discovered module count {len(modules)} is below population floor {MIN_DISCOVERED_MODULES}",
        )
        self.assertGreaterEqual(
            len(bindings),
            MIN_PUBLIC_BINDINGS,
            f"Discovered public binding count {len(bindings)} is below population floor {MIN_PUBLIC_BINDINGS}",
        )

    def test_no_function_local_bindings(self) -> None:
        _, bindings = _collect_lumen_public_bindings()
        self.assertGreaterEqual(
            len(bindings),
            MIN_PUBLIC_BINDINGS,
            f"Population floor {MIN_PUBLIC_BINDINGS} not met for qualname check",
        )
        violations: list[str] = []
        for mod, name, obj in bindings:
            qualname = getattr(obj, "__qualname__", "")
            if "<locals>" in qualname:
                violations.append(f"{mod.__name__}.{name} (qualname={qualname})")
        self.assertEqual(
            violations,
            [],
            f"Public symbols defined inside function body: {violations}",
        )

    def test_declaring_module_identity_consistency(self) -> None:
        _, bindings = _collect_lumen_public_bindings()
        self.assertGreaterEqual(
            len(bindings),
            MIN_PUBLIC_BINDINGS,
            f"Population floor {MIN_PUBLIC_BINDINGS} not met for identity check",
        )
        violations: list[str] = []
        for mod, name, obj in bindings:
            decl_mod_name = getattr(obj, "__module__", None)
            if not decl_mod_name:
                continue
            decl_mod = sys.modules.get(decl_mod_name)
            if decl_mod is None:
                try:
                    decl_mod = importlib.import_module(decl_mod_name)
                except Exception:
                    decl_mod = None
            decl_obj = getattr(decl_mod, name, None) if decl_mod is not None else None
            if decl_obj is not obj:
                violations.append(
                    f"{mod.__name__}.{name}: bound to {obj!r} (declared in {decl_mod_name}), "
                    f"but {decl_mod_name}.{name} is {decl_obj!r}"
                )
        self.assertEqual(
            violations,
            [],
            f"Public symbols whose declaring module does not bind them to the same object: {violations}",
        )


if __name__ == "__main__":
    unittest.main()
