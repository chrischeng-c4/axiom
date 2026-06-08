"""
Lazy loading utilities for test and benchmark modules.

This module provides on-demand module loading, called by the Rust discovery engine
only when tests/benchmarks need to be executed. This avoids upfront loading overhead.
"""

from __future__ import annotations

import importlib.util
import inspect
import sys
from pathlib import Path
from typing import List, Type, Any

from .suite import TestSuite
from .benchmark import BenchmarkGroup


def lazy_load_test_suite(file_path: Path) -> List[Type[TestSuite]]:
    """
    Load a single test file and return TestSuite classes.

    Args:
        file_path: Absolute path to the test file (test_*.py)

    Returns:
        List of TestSuite subclasses found in the module

    Raises:
        ImportError: If module cannot be loaded
        ValueError: If file_path doesn't exist

    Example:
        >>> suites = lazy_load_test_suite(Path("tests/mongo/unit/test_document.py"))
        >>> for suite in suites:
        ...     print(suite.__name__)
    """
    if not file_path.exists():
        raise ValueError(f"Test file does not exist: {file_path}")

    if not file_path.is_file():
        raise ValueError(f"Path is not a file: {file_path}")

    # Generate a unique module name based on file path
    # e.g., "tests.mongo.unit.test_document"
    module_name = _path_to_module_name(file_path)

    # Load the module dynamically
    module = _load_module_from_path(module_name, file_path)

    # Find all TestSuite subclasses in the module
    test_suites = []
    for name, obj in inspect.getmembers(module, inspect.isclass):
        # Check if it's a TestSuite subclass (but not TestSuite itself)
        if issubclass(obj, TestSuite) and obj is not TestSuite:
            # Only include classes defined in this module (not imported)
            if obj.__module__ == module_name:
                test_suites.append(obj)

    return test_suites


def lazy_load_benchmark(file_path: Path) -> List[BenchmarkGroup]:
    """
    Load a single benchmark file and return registered BenchmarkGroup instances.

    Args:
        file_path: Absolute path to the benchmark file (bench_*.py)

    Returns:
        List of BenchmarkGroup instances registered in the module

    Raises:
        ImportError: If module cannot be loaded
        ValueError: If file_path doesn't exist

    Example:
        >>> groups = lazy_load_benchmark(Path("benchmarks/bench_insert.py"))
        >>> for group in groups:
        ...     print(group.name)
    """
    if not file_path.exists():
        raise ValueError(f"Benchmark file does not exist: {file_path}")

    if not file_path.is_file():
        raise ValueError(f"Path is not a file: {file_path}")

    # Generate module name
    module_name = _path_to_module_name(file_path)

    # Load the module dynamically
    # This will trigger register_group() calls in the module
    module = _load_module_from_path(module_name, file_path)

    # Find BenchmarkGroup instances in the module
    # BenchmarkGroups are created as module-level variables in benchmark files
    benchmark_groups = []
    for name, obj in inspect.getmembers(module):
        if isinstance(obj, BenchmarkGroup):
            benchmark_groups.append(obj)

    return benchmark_groups


def _path_to_module_name(file_path: Path) -> str:
    """
    Convert a file path to a Python module name.

    Args:
        file_path: Absolute or relative path to a Python file

    Returns:
        Module name (e.g., "tests.mongo.unit.test_document")

    Examples:
        >>> _path_to_module_name(Path("tests/mongo/unit/test_document.py"))
        'tests.mongo.unit.test_document'
        >>> _path_to_module_name(Path("benchmarks/bench_insert.py"))
        'benchmarks.bench_insert'
    """
    # Convert to absolute path
    abs_path = file_path.resolve()

    # Try to find the project root (where pyproject.toml is)
    current = abs_path.parent
    while current != current.parent:
        if (current / "pyproject.toml").exists():
            # Found project root
            try:
                rel_path = abs_path.relative_to(current)
            except ValueError:
                # File is outside project root, use absolute path
                rel_path = abs_path
            break
        current = current.parent
    else:
        # Didn't find project root, use the file's parent as base
        rel_path = abs_path.relative_to(abs_path.parent.parent)

    # Convert path to module name
    # e.g., "tests/mongo/unit/test_document.py" -> "tests.mongo.unit.test_document"
    module_parts = list(rel_path.parts[:-1]) + [rel_path.stem]
    module_name = ".".join(module_parts)

    return module_name


def _load_module_from_path(module_name: str, file_path: Path) -> Any:
    """
    Dynamically load a Python module from a file path.

    Args:
        module_name: Full module name (e.g., "tests.mongo.unit.test_document")
        file_path: Absolute path to the Python file

    Returns:
        Loaded module object

    Raises:
        ImportError: If module cannot be loaded
    """
    # Check if module is already loaded
    if module_name in sys.modules:
        # Reload to ensure fresh state
        return sys.modules[module_name]

    # Add parent directories to sys.path for imports
    # Find the root directory (where the module hierarchy starts)
    abs_file_path = file_path.resolve()
    module_parts = module_name.split(".")

    # Calculate the root directory by going up from the file path
    # based on the number of parts in the module name
    root_dir = abs_file_path.parent
    for _ in range(len(module_parts) - 1):
        root_dir = root_dir.parent

    # Add root directory to sys.path temporarily
    path_added = False
    if str(root_dir) not in sys.path:
        sys.path.insert(0, str(root_dir))
        path_added = True

    try:
        # Create module spec from file
        spec = importlib.util.spec_from_file_location(module_name, file_path)
        if spec is None or spec.loader is None:
            raise ImportError(f"Cannot create module spec for {file_path}")

        # Create module from spec
        module = importlib.util.module_from_spec(spec)

        # Add to sys.modules before execution (allows relative imports)
        sys.modules[module_name] = module

        try:
            # Execute module code
            spec.loader.exec_module(module)
        except Exception as e:
            # Remove from sys.modules on failure
            sys.modules.pop(module_name, None)
            raise ImportError(f"Failed to load module {module_name} from {file_path}: {e}") from e

        return module

    finally:
        # Clean up sys.path if we added the root directory
        if path_added and str(root_dir) in sys.path:
            sys.path.remove(str(root_dir))


def unload_module(module_name: str) -> None:
    """
    Unload a previously loaded module.

    Useful for cleanup or forcing a fresh reload.

    Args:
        module_name: Full module name to unload
    """
    if module_name in sys.modules:
        del sys.modules[module_name]


def lazy_load_test_module(file_path: Path) -> Any:
    """
    Generic module loader for test files.

    Loads the module without inspecting classes. Useful when you want
    the raw module object for custom processing.

    Args:
        file_path: Absolute path to the Python file

    Returns:
        Loaded module object
    """
    module_name = _path_to_module_name(file_path)
    return _load_module_from_path(module_name, file_path)
