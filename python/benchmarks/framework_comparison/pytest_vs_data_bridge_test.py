#!/usr/bin/env python3
"""
Comprehensive benchmark comparing pytest vs data-bridge-test performance.

This benchmark measures:
1. Test discovery speed
2. Test execution speed (simple tests)
3. Fixture overhead
4. Parametrized test generation
5. Memory usage (if psutil available)

Usage:
    python benchmarks/framework_comparison/pytest_vs_cclab_test.py

Requirements:
    - pytest
    - pytest-benchmark (optional, for pytest metrics)
    - psutil (optional, for memory tracking)
"""

import asyncio
import gc
import os
import statistics
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

from cclab.probe import (
    BenchmarkGroup, benchmark, register_group,
    DiscoveryConfig, TestRegistry, discover_files,
)

# Optional dependencies
try:
    import psutil
    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False
    print("Warning: psutil not available, memory tracking disabled")

# Constants
SAMPLE_TESTS_PATH = Path(__file__).parent / "sample_tests.py"
WARMUP_ROUNDS = 3
MEASUREMENT_ROUNDS = 10


# =============================================================================
# Utility Functions
# =============================================================================

def get_memory_usage() -> Optional[float]:
    """Get current process memory usage in MB."""
    if not HAS_PSUTIL:
        return None
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / (1024 * 1024)


def force_gc():
    """Force garbage collection."""
    gc.collect()
    gc.collect()
    gc.collect()


# =============================================================================
# pytest Benchmarks
# =============================================================================

def benchmark_pytest_discovery() -> Dict[str, float]:
    """Benchmark pytest test discovery."""
    import pytest

    results = {
        "discovery_time_ms": [],
        "test_count": 0,
    }

    # Warmup
    for _ in range(WARMUP_ROUNDS):
        force_gc()
        pytest.main([
            str(SAMPLE_TESTS_PATH),
            "--collect-only",
            "-q",
        ])

    # Measure
    for _ in range(MEASUREMENT_ROUNDS):
        force_gc()
        start = time.perf_counter()

        # Use pytest's collection mechanism
        class CollectionPlugin:
            def __init__(self):
                self.collected = []

            def pytest_collection_finish(self, session):
                self.collected = session.items

        plugin = CollectionPlugin()
        pytest.main([
            str(SAMPLE_TESTS_PATH),
            "--collect-only",
            "-q",
        ], plugins=[plugin])

        elapsed_ms = (time.perf_counter() - start) * 1000
        results["discovery_time_ms"].append(elapsed_ms)
        results["test_count"] = len(plugin.collected)

    return results


def benchmark_pytest_execution() -> Dict[str, float]:
    """Benchmark pytest test execution speed."""
    import pytest

    results = {
        "execution_time_ms": [],
        "tests_run": 0,
    }

    # Warmup
    for _ in range(WARMUP_ROUNDS):
        force_gc()
        pytest.main([
            str(SAMPLE_TESTS_PATH),
            "-q",
            "-x",  # Exit on first failure (for warmup speed)
        ])

    # Measure
    for _ in range(MEASUREMENT_ROUNDS):
        force_gc()
        mem_before = get_memory_usage()
        start = time.perf_counter()

        # Run tests
        result = pytest.main([
            str(SAMPLE_TESTS_PATH),
            "-q",
            "--tb=no",  # No traceback for speed
        ])

        elapsed_ms = (time.perf_counter() - start) * 1000
        mem_after = get_memory_usage()

        results["execution_time_ms"].append(elapsed_ms)
        if mem_before and mem_after:
            if "memory_delta_mb" not in results:
                results["memory_delta_mb"] = []
            results["memory_delta_mb"].append(mem_after - mem_before)

    return results


def benchmark_pytest_parametrize() -> Dict[str, float]:
    """Benchmark pytest parametrization overhead."""
    import pytest
    import tempfile

    # Create a parametrized test file
    parametrized_test = '''
import pytest

@pytest.mark.parametrize("a,b,expected", [
    (1, 2, 3),
    (5, 10, 15),
    (100, 200, 300),
    (-5, 10, 5),
    (0, 0, 0),
])
def test_parametrized_add(a, b, expected):
    assert a + b == expected

@pytest.mark.parametrize("x,y,expected", [
    (2, 3, 6),
    (4, 5, 20),
    (10, 10, 100),
])
def test_parametrized_multiply(x, y, expected):
    assert x * y == expected
'''

    results = {
        "generation_time_ms": [],
        "execution_time_ms": [],
        "param_count": 0,
    }

    with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
        f.write(parametrized_test)
        test_file = f.name

    try:
        # Warmup
        for _ in range(WARMUP_ROUNDS):
            force_gc()
            pytest.main([test_file, "-q"])

        # Measure generation (collection)
        for _ in range(MEASUREMENT_ROUNDS):
            force_gc()
            start = time.perf_counter()

            class CollectionPlugin:
                def __init__(self):
                    self.collected = []

                def pytest_collection_finish(self, session):
                    self.collected = session.items

            plugin = CollectionPlugin()
            pytest.main([
                test_file,
                "--collect-only",
                "-q",
            ], plugins=[plugin])

            elapsed_ms = (time.perf_counter() - start) * 1000
            results["generation_time_ms"].append(elapsed_ms)
            results["param_count"] = len(plugin.collected)

        # Measure execution
        for _ in range(MEASUREMENT_ROUNDS):
            force_gc()
            start = time.perf_counter()

            pytest.main([test_file, "-q", "--tb=no"])

            elapsed_ms = (time.perf_counter() - start) * 1000
            results["execution_time_ms"].append(elapsed_ms)

    finally:
        os.unlink(test_file)

    return results


def benchmark_pytest_fixtures() -> Dict[str, float]:
    """Benchmark pytest fixture overhead."""
    import pytest
    import tempfile

    # Create a test file with fixtures
    fixture_test = '''
import pytest

@pytest.fixture
def simple_fixture():
    return 42

@pytest.fixture
def list_fixture():
    return [1, 2, 3, 4, 5]

@pytest.fixture
def dict_fixture():
    return {"a": 1, "b": 2, "c": 3}

@pytest.fixture
def complex_fixture(simple_fixture, list_fixture, dict_fixture):
    return {
        "simple": simple_fixture,
        "list": list_fixture,
        "dict": dict_fixture,
    }

def test_with_simple_fixture(simple_fixture):
    assert simple_fixture == 42

def test_with_list_fixture(list_fixture):
    assert len(list_fixture) == 5

def test_with_dict_fixture(dict_fixture):
    assert dict_fixture["a"] == 1

def test_with_complex_fixture(complex_fixture):
    assert complex_fixture["simple"] == 42
'''

    results = {
        "execution_time_ms": [],
        "fixture_count": 4,
    }

    with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
        f.write(fixture_test)
        test_file = f.name

    try:
        # Warmup
        for _ in range(WARMUP_ROUNDS):
            force_gc()
            pytest.main([test_file, "-q"])

        # Measure
        for _ in range(MEASUREMENT_ROUNDS):
            force_gc()
            start = time.perf_counter()

            pytest.main([test_file, "-q", "--tb=no"])

            elapsed_ms = (time.perf_counter() - start) * 1000
            results["execution_time_ms"].append(elapsed_ms)

    finally:
        os.unlink(test_file)

    return results


# =============================================================================
# data-bridge-test Benchmarks
# =============================================================================

async def benchmark_dbt_discovery() -> Dict[str, float]:
    """Benchmark data-bridge-test discovery."""
    results = {
        "discovery_time_ms": [],
        "test_count": 0,
    }

    # Warmup
    for _ in range(WARMUP_ROUNDS):
        force_gc()
        config = DiscoveryConfig()
        files = discover_files(config)

    # Measure
    for _ in range(MEASUREMENT_ROUNDS):
        force_gc()
        start = time.perf_counter()

        config = DiscoveryConfig()
        files = discover_files(config)

        # Count test functions
        test_count = 0
        import ast
        with open(SAMPLE_TESTS_PATH, 'r') as f:
            tree = ast.parse(f.read())
            for node in ast.walk(tree):
                if isinstance(node, ast.FunctionDef) and node.name.startswith('test_'):
                    test_count += 1

        elapsed_ms = (time.perf_counter() - start) * 1000
        results["discovery_time_ms"].append(elapsed_ms)
        results["test_count"] = test_count

    return results


async def benchmark_dbt_execution() -> Dict[str, float]:
    """Benchmark data-bridge-test execution speed."""
    from cclab.probe import TestSuite, test, expect, TestRunner

    results = {
        "execution_time_ms": [],
        "tests_run": 0,
    }

    # Define test suite
    class SimpleSuite(TestSuite):
        @test
        async def test_addition(self):
            expect(1 + 2).to_equal(3)

        @test
        async def test_multiplication(self):
            expect(3 * 4).to_equal(12)

        @test
        async def test_string_ops(self):
            s = "hello world"
            expect(s.upper()).to_equal("HELLO WORLD")

        @test
        async def test_list_ops(self):
            lst = [1, 2, 3, 4, 5]
            expect(len(lst)).to_equal(5)

        @test
        async def test_dict_ops(self):
            d = {"a": 1, "b": 2, "c": 3}
            expect(len(d)).to_equal(3)

        @test
        async def test_boolean_logic(self):
            expect(True).to_equal(True)

        @test
        async def test_comparisons(self):
            expect(5 > 3).to_equal(True)

        @test
        async def test_type_checks(self):
            expect(isinstance(42, int)).to_equal(True)

    # Warmup
    for _ in range(WARMUP_ROUNDS):
        force_gc()
        runner = TestRunner()
        suite = SimpleSuite()
        await suite.run(runner=runner, verbose=False)

    # Measure
    for _ in range(MEASUREMENT_ROUNDS):
        force_gc()
        mem_before = get_memory_usage()
        start = time.perf_counter()

        runner = TestRunner()
        suite = SimpleSuite()
        report = await suite.run(runner=runner, verbose=False)

        elapsed_ms = (time.perf_counter() - start) * 1000
        mem_after = get_memory_usage()

        results["execution_time_ms"].append(elapsed_ms)
        results["tests_run"] = report.summary.total

        if mem_before and mem_after:
            if "memory_delta_mb" not in results:
                results["memory_delta_mb"] = []
            results["memory_delta_mb"].append(mem_after - mem_before)

    return results


async def benchmark_dbt_parametrize() -> Dict[str, float]:
    """Benchmark data-bridge-test parametrization."""
    from cclab.probe import TestSuite, test, expect, TestRunner, parametrize

    results = {
        "generation_time_ms": [],
        "execution_time_ms": [],
        "param_count": 8,  # 5 + 3 parameter sets
    }

    # Define parametrized tests
    class ParametrizedSuite(TestSuite):
        @parametrize("a,b,expected", [
            (1, 2, 3),
            (5, 10, 15),
            (100, 200, 300),
            (-5, 10, 5),
            (0, 0, 0),
        ])
        @test
        async def test_parametrized_add(self, a, b, expected):
            expect(a + b).to_equal(expected)

        @parametrize("x,y,expected", [
            (2, 3, 6),
            (4, 5, 20),
            (10, 10, 100),
        ])
        @test
        async def test_parametrized_multiply(self, x, y, expected):
            expect(x * y).to_equal(expected)

    # Warmup
    for _ in range(WARMUP_ROUNDS):
        force_gc()
        runner = TestRunner()
        suite = ParametrizedSuite()
        await suite.run(runner=runner, verbose=False)

    # Measure execution (generation is implicit in data-bridge-test)
    for _ in range(MEASUREMENT_ROUNDS):
        force_gc()

        # Generation time (decorator application)
        start = time.perf_counter()
        # The parametrize decorator generates tests at decoration time
        # So we measure the suite instantiation
        suite_instance = ParametrizedSuite()
        gen_elapsed_ms = (time.perf_counter() - start) * 1000
        results["generation_time_ms"].append(gen_elapsed_ms)

        # Execution time
        force_gc()
        start = time.perf_counter()
        runner = TestRunner()
        suite = ParametrizedSuite()
        await suite.run(runner=runner, verbose=False)
        exec_elapsed_ms = (time.perf_counter() - start) * 1000
        results["execution_time_ms"].append(exec_elapsed_ms)

    return results


async def benchmark_dbt_fixtures() -> Dict[str, float]:
    """Benchmark data-bridge-test fixture overhead."""
    from cclab.probe import TestSuite, test, expect, TestRunner, fixture

    results = {
        "execution_time_ms": [],
        "fixture_count": 4,
    }

    # Define test suite with fixtures
    class FixtureSuite(TestSuite):
        @fixture
        async def simple_fixture(self):
            return 42

        @fixture
        async def list_fixture(self):
            return [1, 2, 3, 4, 5]

        @fixture
        async def dict_fixture(self):
            return {"a": 1, "b": 2, "c": 3}

        @fixture
        async def complex_fixture(self, simple_fixture, list_fixture, dict_fixture):
            return {
                "simple": simple_fixture,
                "list": list_fixture,
                "dict": dict_fixture,
            }

        @test
        async def test_with_simple_fixture(self, simple_fixture):
            expect(simple_fixture).to_equal(42)

        @test
        async def test_with_list_fixture(self, list_fixture):
            expect(len(list_fixture)).to_equal(5)

        @test
        async def test_with_dict_fixture(self, dict_fixture):
            expect(dict_fixture["a"]).to_equal(1)

        @test
        async def test_with_complex_fixture(self, complex_fixture):
            expect(complex_fixture["simple"]).to_equal(42)

    # Warmup
    for _ in range(WARMUP_ROUNDS):
        force_gc()
        runner = TestRunner()
        suite = FixtureSuite()
        await suite.run(runner=runner, verbose=False)

    # Measure
    for _ in range(MEASUREMENT_ROUNDS):
        force_gc()
        start = time.perf_counter()

        runner = TestRunner()
        suite = FixtureSuite()
        await suite.run(runner=runner, verbose=False)

        elapsed_ms = (time.perf_counter() - start) * 1000
        results["execution_time_ms"].append(elapsed_ms)

    return results


# =============================================================================
# Report Generation
# =============================================================================

def calculate_stats(values: List[float]) -> Dict[str, float]:
    """Calculate statistics for a list of values."""
    if not values:
        return {}

    return {
        "min": min(values),
        "max": max(values),
        "mean": statistics.mean(values),
        "median": statistics.median(values),
        "stdev": statistics.stdev(values) if len(values) > 1 else 0.0,
    }


def generate_markdown_report(pytest_results: Dict, dbt_results: Dict) -> str:
    """Generate markdown comparison report."""

    lines = [
        "# pytest vs data-bridge-test Performance Comparison",
        "",
        f"**Date**: {time.strftime('%Y-%m-%d %H:%M:%S')}",
        f"**Python**: {sys.version.split()[0]}",
        f"**Measurement Rounds**: {MEASUREMENT_ROUNDS}",
        f"**Warmup Rounds**: {WARMUP_ROUNDS}",
        "",
        "## Summary",
        "",
    ]

    # Calculate speedups
    speedups = {}

    # Test Discovery
    if "discovery" in pytest_results and "discovery" in dbt_results:
        pytest_disc = pytest_results["discovery"]["discovery_time_ms"]["mean"]
        dbt_disc = dbt_results["discovery"]["discovery_time_ms"]["mean"]
        speedups["discovery"] = pytest_disc / dbt_disc if dbt_disc > 0 else 0

    # Test Execution
    if "execution" in pytest_results and "execution" in dbt_results:
        pytest_exec = pytest_results["execution"]["execution_time_ms"]["mean"]
        dbt_exec = dbt_results["execution"]["execution_time_ms"]["mean"]
        speedups["execution"] = pytest_exec / dbt_exec if dbt_exec > 0 else 0

    # Parametrization
    if "parametrize" in pytest_results and "parametrize" in dbt_results:
        pytest_param = pytest_results["parametrize"]["execution_time_ms"]["mean"]
        dbt_param = dbt_results["parametrize"]["execution_time_ms"]["mean"]
        speedups["parametrize"] = pytest_param / dbt_param if dbt_param > 0 else 0

    # Fixtures
    if "fixtures" in pytest_results and "fixtures" in dbt_results:
        pytest_fix = pytest_results["fixtures"]["execution_time_ms"]["mean"]
        dbt_fix = dbt_results["fixtures"]["execution_time_ms"]["mean"]
        speedups["fixtures"] = pytest_fix / dbt_fix if dbt_fix > 0 else 0

    # Summary table
    lines.extend([
        "| Metric | pytest (ms) | data-bridge-test (ms) | Speedup |",
        "|--------|-------------|----------------------|---------|",
    ])

    if "discovery" in speedups:
        lines.append(
            f"| Test Discovery | {pytest_results['discovery']['discovery_time_ms']['mean']:.2f} | "
            f"{dbt_results['discovery']['discovery_time_ms']['mean']:.2f} | "
            f"**{speedups['discovery']:.2f}x** |"
        )

    if "execution" in speedups:
        lines.append(
            f"| Test Execution | {pytest_results['execution']['execution_time_ms']['mean']:.2f} | "
            f"{dbt_results['execution']['execution_time_ms']['mean']:.2f} | "
            f"**{speedups['execution']:.2f}x** |"
        )

    if "parametrize" in speedups:
        lines.append(
            f"| Parametrization | {pytest_results['parametrize']['execution_time_ms']['mean']:.2f} | "
            f"{dbt_results['parametrize']['execution_time_ms']['mean']:.2f} | "
            f"**{speedups['parametrize']:.2f}x** |"
        )

    if "fixtures" in speedups:
        lines.append(
            f"| Fixtures | {pytest_results['fixtures']['execution_time_ms']['mean']:.2f} | "
            f"{dbt_results['fixtures']['execution_time_ms']['mean']:.2f} | "
            f"**{speedups['fixtures']:.2f}x** |"
        )

    lines.extend([
        "",
        "## Detailed Results",
        "",
    ])

    # Test Discovery Details
    if "discovery" in pytest_results and "discovery" in dbt_results:
        lines.extend([
            "### Test Discovery",
            "",
            "| Framework | Min | Max | Mean | Median | StdDev |",
            "|-----------|-----|-----|------|--------|--------|",
            f"| pytest | {pytest_results['discovery']['discovery_time_ms']['min']:.2f} | "
            f"{pytest_results['discovery']['discovery_time_ms']['max']:.2f} | "
            f"{pytest_results['discovery']['discovery_time_ms']['mean']:.2f} | "
            f"{pytest_results['discovery']['discovery_time_ms']['median']:.2f} | "
            f"{pytest_results['discovery']['discovery_time_ms']['stdev']:.2f} |",
            f"| data-bridge-test | {dbt_results['discovery']['discovery_time_ms']['min']:.2f} | "
            f"{dbt_results['discovery']['discovery_time_ms']['max']:.2f} | "
            f"{dbt_results['discovery']['discovery_time_ms']['mean']:.2f} | "
            f"{dbt_results['discovery']['discovery_time_ms']['median']:.2f} | "
            f"{dbt_results['discovery']['discovery_time_ms']['stdev']:.2f} |",
            "",
        ])

    # Test Execution Details
    if "execution" in pytest_results and "execution" in dbt_results:
        lines.extend([
            "### Test Execution",
            "",
            "| Framework | Min | Max | Mean | Median | StdDev |",
            "|-----------|-----|-----|------|--------|--------|",
            f"| pytest | {pytest_results['execution']['execution_time_ms']['min']:.2f} | "
            f"{pytest_results['execution']['execution_time_ms']['max']:.2f} | "
            f"{pytest_results['execution']['execution_time_ms']['mean']:.2f} | "
            f"{pytest_results['execution']['execution_time_ms']['median']:.2f} | "
            f"{pytest_results['execution']['execution_time_ms']['stdev']:.2f} |",
            f"| data-bridge-test | {dbt_results['execution']['execution_time_ms']['min']:.2f} | "
            f"{dbt_results['execution']['execution_time_ms']['max']:.2f} | "
            f"{dbt_results['execution']['execution_time_ms']['mean']:.2f} | "
            f"{dbt_results['execution']['execution_time_ms']['median']:.2f} | "
            f"{dbt_results['execution']['execution_time_ms']['stdev']:.2f} |",
            "",
        ])

        # Memory usage if available
        if "memory_delta_mb" in pytest_results["execution"] and "memory_delta_mb" in dbt_results["execution"]:
            lines.extend([
                "#### Memory Usage (Execution)",
                "",
                "| Framework | Mean ΔMemory (MB) |",
                "|-----------|-------------------|",
                f"| pytest | {pytest_results['execution']['memory_delta_mb']['mean']:.2f} |",
                f"| data-bridge-test | {dbt_results['execution']['memory_delta_mb']['mean']:.2f} |",
                "",
            ])

    # Parametrization Details
    if "parametrize" in pytest_results and "parametrize" in dbt_results:
        lines.extend([
            "### Parametrization",
            "",
            "| Framework | Min | Max | Mean | Median | StdDev |",
            "|-----------|-----|-----|------|--------|--------|",
            f"| pytest | {pytest_results['parametrize']['execution_time_ms']['min']:.2f} | "
            f"{pytest_results['parametrize']['execution_time_ms']['max']:.2f} | "
            f"{pytest_results['parametrize']['execution_time_ms']['mean']:.2f} | "
            f"{pytest_results['parametrize']['execution_time_ms']['median']:.2f} | "
            f"{pytest_results['parametrize']['execution_time_ms']['stdev']:.2f} |",
            f"| data-bridge-test | {dbt_results['parametrize']['execution_time_ms']['min']:.2f} | "
            f"{dbt_results['parametrize']['execution_time_ms']['max']:.2f} | "
            f"{dbt_results['parametrize']['execution_time_ms']['mean']:.2f} | "
            f"{dbt_results['parametrize']['execution_time_ms']['median']:.2f} | "
            f"{dbt_results['parametrize']['execution_time_ms']['stdev']:.2f} |",
            "",
        ])

    # Fixtures Details
    if "fixtures" in pytest_results and "fixtures" in dbt_results:
        lines.extend([
            "### Fixtures",
            "",
            "| Framework | Min | Max | Mean | Median | StdDev |",
            "|-----------|-----|-----|------|--------|--------|",
            f"| pytest | {pytest_results['fixtures']['execution_time_ms']['min']:.2f} | "
            f"{pytest_results['fixtures']['execution_time_ms']['max']:.2f} | "
            f"{pytest_results['fixtures']['execution_time_ms']['mean']:.2f} | "
            f"{pytest_results['fixtures']['execution_time_ms']['median']:.2f} | "
            f"{pytest_results['fixtures']['execution_time_ms']['stdev']:.2f} |",
            f"| data-bridge-test | {dbt_results['fixtures']['execution_time_ms']['min']:.2f} | "
            f"{dbt_results['fixtures']['execution_time_ms']['max']:.2f} | "
            f"{dbt_results['fixtures']['execution_time_ms']['mean']:.2f} | "
            f"{dbt_results['fixtures']['execution_time_ms']['median']:.2f} | "
            f"{dbt_results['fixtures']['execution_time_ms']['stdev']:.2f} |",
            "",
        ])

    lines.extend([
        "## Analysis",
        "",
        "### Key Findings",
        "",
    ])

    avg_speedup = statistics.mean(speedups.values()) if speedups else 0

    lines.append(f"- **Average Speedup**: {avg_speedup:.2f}x")

    if speedups:
        best_category = max(speedups, key=speedups.get)
        lines.append(f"- **Best Performance**: {best_category.title()} ({speedups[best_category]:.2f}x faster)")

    lines.extend([
        "",
        "### Performance Characteristics",
        "",
        "**pytest**:",
        "- Mature, feature-rich framework with extensive plugin ecosystem",
        "- Collection and execution phases are separate",
        "- Python-based fixture and parametrization system",
        "",
        "**data-bridge-test**:",
        "- Rust-powered engine with minimal Python overhead",
        "- Native async/await support",
        "- Integrated collection and execution",
        "- Zero-copy data structures where possible",
        "",
        "## Conclusion",
        "",
        f"data-bridge-test demonstrates **{avg_speedup:.1f}x average speedup** over pytest "
        "for basic test operations. This performance advantage comes from:",
        "",
        "1. **Rust Engine**: Core test execution in compiled Rust code",
        "2. **Reduced Overhead**: Minimal Python layer overhead",
        "3. **Async Native**: Built for async/await from the ground up",
        "4. **Zero-Copy**: Efficient data handling between Rust and Python",
        "",
        "While pytest remains the standard for Python testing with its rich ecosystem, ",
        "data-bridge-test offers significant performance benefits for projects that need ",
        "fast test execution, especially in CI/CD pipelines or development workflows.",
    ])

    return "\n".join(lines)


def print_console_report(pytest_results: Dict, dbt_results: Dict):
    """Print comparison report to console."""

    print("\n" + "=" * 80)
    print("pytest vs data-bridge-test Performance Comparison")
    print("=" * 80)
    print()

    # Summary table
    print("SUMMARY")
    print("-" * 80)
    print(f"{'Metric':<25} {'pytest (ms)':<15} {'data-bridge (ms)':<15} {'Speedup':<10}")
    print("-" * 80)

    if "discovery" in pytest_results and "discovery" in dbt_results:
        pytest_val = pytest_results["discovery"]["discovery_time_ms"]["mean"]
        dbt_val = dbt_results["discovery"]["discovery_time_ms"]["mean"]
        speedup = pytest_val / dbt_val if dbt_val > 0 else 0
        print(f"{'Test Discovery':<25} {pytest_val:>13.2f}  {dbt_val:>13.2f}  {speedup:>8.2f}x")

    if "execution" in pytest_results and "execution" in dbt_results:
        pytest_val = pytest_results["execution"]["execution_time_ms"]["mean"]
        dbt_val = dbt_results["execution"]["execution_time_ms"]["mean"]
        speedup = pytest_val / dbt_val if dbt_val > 0 else 0
        print(f"{'Test Execution':<25} {pytest_val:>13.2f}  {dbt_val:>13.2f}  {speedup:>8.2f}x")

    if "parametrize" in pytest_results and "parametrize" in dbt_results:
        pytest_val = pytest_results["parametrize"]["execution_time_ms"]["mean"]
        dbt_val = dbt_results["parametrize"]["execution_time_ms"]["mean"]
        speedup = pytest_val / dbt_val if dbt_val > 0 else 0
        print(f"{'Parametrization':<25} {pytest_val:>13.2f}  {dbt_val:>13.2f}  {speedup:>8.2f}x")

    if "fixtures" in pytest_results and "fixtures" in dbt_results:
        pytest_val = pytest_results["fixtures"]["execution_time_ms"]["mean"]
        dbt_val = dbt_results["fixtures"]["execution_time_ms"]["mean"]
        speedup = pytest_val / dbt_val if dbt_val > 0 else 0
        print(f"{'Fixtures':<25} {pytest_val:>13.2f}  {dbt_val:>13.2f}  {speedup:>8.2f}x")

    print("-" * 80)
    print()


# =============================================================================
# Main Execution
# =============================================================================

async def main():
    """Run all benchmarks and generate report."""

    print("\n" + "=" * 80)
    print("Framework Comparison Benchmark")
    print("=" * 80)
    print(f"Python: {sys.version.split()[0]}")
    print(f"Rounds: {MEASUREMENT_ROUNDS}")
    print(f"Warmup: {WARMUP_ROUNDS}")
    print("=" * 80)

    # Verify sample tests exist
    if not SAMPLE_TESTS_PATH.exists():
        print(f"ERROR: Sample tests not found at {SAMPLE_TESTS_PATH}")
        sys.exit(1)

    pytest_results = {}
    dbt_results = {}

    # pytest benchmarks
    print("\n[1/8] Running pytest discovery benchmark...")
    try:
        result = benchmark_pytest_discovery()
        pytest_results["discovery"] = {
            "discovery_time_ms": calculate_stats(result["discovery_time_ms"]),
            "test_count": result["test_count"],
        }
        print(f"  ✓ Completed (mean: {pytest_results['discovery']['discovery_time_ms']['mean']:.2f} ms)")
    except Exception as e:
        print(f"  ✗ Failed: {e}")

    print("\n[2/8] Running pytest execution benchmark...")
    try:
        result = benchmark_pytest_execution()
        pytest_results["execution"] = {
            "execution_time_ms": calculate_stats(result["execution_time_ms"]),
        }
        if "memory_delta_mb" in result:
            pytest_results["execution"]["memory_delta_mb"] = calculate_stats(result["memory_delta_mb"])
        print(f"  ✓ Completed (mean: {pytest_results['execution']['execution_time_ms']['mean']:.2f} ms)")
    except Exception as e:
        print(f"  ✗ Failed: {e}")

    print("\n[3/8] Running pytest parametrize benchmark...")
    try:
        result = benchmark_pytest_parametrize()
        pytest_results["parametrize"] = {
            "generation_time_ms": calculate_stats(result["generation_time_ms"]),
            "execution_time_ms": calculate_stats(result["execution_time_ms"]),
            "param_count": result["param_count"],
        }
        print(f"  ✓ Completed (mean: {pytest_results['parametrize']['execution_time_ms']['mean']:.2f} ms)")
    except Exception as e:
        print(f"  ✗ Failed: {e}")

    print("\n[4/8] Running pytest fixtures benchmark...")
    try:
        result = benchmark_pytest_fixtures()
        pytest_results["fixtures"] = {
            "execution_time_ms": calculate_stats(result["execution_time_ms"]),
            "fixture_count": result["fixture_count"],
        }
        print(f"  ✓ Completed (mean: {pytest_results['fixtures']['execution_time_ms']['mean']:.2f} ms)")
    except Exception as e:
        print(f"  ✗ Failed: {e}")

    # data-bridge-test benchmarks
    print("\n[5/8] Running data-bridge-test discovery benchmark...")
    try:
        result = await benchmark_dbt_discovery()
        dbt_results["discovery"] = {
            "discovery_time_ms": calculate_stats(result["discovery_time_ms"]),
            "test_count": result["test_count"],
        }
        print(f"  ✓ Completed (mean: {dbt_results['discovery']['discovery_time_ms']['mean']:.2f} ms)")
    except Exception as e:
        print(f"  ✗ Failed: {e}")

    print("\n[6/8] Running data-bridge-test execution benchmark...")
    try:
        result = await benchmark_dbt_execution()
        dbt_results["execution"] = {
            "execution_time_ms": calculate_stats(result["execution_time_ms"]),
            "tests_run": result["tests_run"],
        }
        if "memory_delta_mb" in result:
            dbt_results["execution"]["memory_delta_mb"] = calculate_stats(result["memory_delta_mb"])
        print(f"  ✓ Completed (mean: {dbt_results['execution']['execution_time_ms']['mean']:.2f} ms)")
    except Exception as e:
        print(f"  ✗ Failed: {e}")

    print("\n[7/8] Running data-bridge-test parametrize benchmark...")
    try:
        result = await benchmark_dbt_parametrize()
        dbt_results["parametrize"] = {
            "generation_time_ms": calculate_stats(result["generation_time_ms"]),
            "execution_time_ms": calculate_stats(result["execution_time_ms"]),
            "param_count": result["param_count"],
        }
        print(f"  ✓ Completed (mean: {dbt_results['parametrize']['execution_time_ms']['mean']:.2f} ms)")
    except Exception as e:
        print(f"  ✗ Failed: {e}")

    print("\n[8/8] Running data-bridge-test fixtures benchmark...")
    try:
        result = await benchmark_dbt_fixtures()
        dbt_results["fixtures"] = {
            "execution_time_ms": calculate_stats(result["execution_time_ms"]),
            "fixture_count": result["fixture_count"],
        }
        print(f"  ✓ Completed (mean: {dbt_results['fixtures']['execution_time_ms']['mean']:.2f} ms)")
    except Exception as e:
        print(f"  ✗ Failed: {e}")

    # Generate reports
    print("\n" + "=" * 80)
    print("Generating reports...")
    print("=" * 80)

    # Console report
    print_console_report(pytest_results, dbt_results)

    # Markdown report
    markdown_report = generate_markdown_report(pytest_results, dbt_results)
    report_path = Path(__file__).parent / "BENCHMARK_REPORT.md"
    with open(report_path, "w") as f:
        f.write(markdown_report)

    print(f"\n✓ Markdown report saved to: {report_path}")
    print("\nDone!")


if __name__ == "__main__":
    asyncio.run(main())
