"""
cclab.qc - Custom Python test framework with Rust engine

A custom test framework (NOT pytest-compatible) providing:
- Unit testing with decorator-based syntax
- Custom assertion API (expect-style)
- Benchmarking with timing statistics
- Profiling (CPU, memory, Rust-Python boundary)
- Stress testing (Tokio-powered concurrency)
- Security testing (fuzzing, injection detection)

Example:
    from cclab.qc import TestSuite, test, expect

    class MyTests(TestSuite):
        @test(timeout=5.0, tags=["unit"])
        async def test_example(self):
            expect(1 + 1).to_equal(2)

Benchmark example:
    from cclab.qc import benchmark, compare_benchmarks

    result = await benchmark(my_async_func, name="my_operation", iterations=100)
    print(result.format())
"""

from __future__ import annotations

# Import from Rust bindings (cclab-qc-pyo3 → _probe module)
try:
    from .. import _probe as _test  # type: ignore
except (ImportError, AttributeError):
    _test = None

TestType = _test.TestType
TestStatus = _test.TestStatus
ReportFormat = _test.ReportFormat
TestMeta = _test.TestMeta
TestResult = _test.TestResult
TestSummary = _test.TestSummary
TestRunner = _test.TestRunner
Expectation = _test.Expectation
expect = _test.expect
Reporter = _test.Reporter
TestReport = _test.TestReport

# Benchmark types from Rust
BenchmarkStats = _test.BenchmarkStats
BenchmarkResult = _test.BenchmarkResult
BenchmarkConfig = _test.BenchmarkConfig
compare_benchmarks = _test.compare_benchmarks
print_comparison_table = _test.print_comparison_table

# Benchmark report types from Rust
BenchmarkEnvironment = _test.BenchmarkEnvironment
BenchmarkReportGroup = _test.BenchmarkReportGroup
BenchmarkReport = _test.BenchmarkReport

# Coverage types from Rust
FileCoverage = _test.FileCoverage
CoverageInfo = _test.CoverageInfo

# Test server from Rust
TestServer = _test.TestServer
TestServerHandle = _test.TestServerHandle

# Discovery types from Rust
FileType = _test.FileType
FileInfo = _test.FileInfo
DiscoveryConfig = _test.DiscoveryConfig
TestRegistry = _test.TestRegistry
BenchmarkRegistry = _test.BenchmarkRegistry
DiscoveryStats = _test.DiscoveryStats
discover_files = _test.discover_files
filter_files_by_pattern = _test.filter_files_by_pattern

# Profiler types from Rust
ProfilePhase = _test.ProfilePhase
PhaseTiming = _test.PhaseTiming
PhaseBreakdown = _test.PhaseBreakdown
GilTestConfig = _test.GilTestConfig
GilContentionResult = _test.GilContentionResult
MemorySnapshot = _test.MemorySnapshot
MemoryProfile = _test.MemoryProfile
FlamegraphData = _test.FlamegraphData
ProfileResult = _test.ProfileResult
ProfileConfig = _test.ProfileConfig
generate_flamegraph = _test.generate_flamegraph

# Fixture types from Rust
FixtureScope = _test.FixtureScope
FixtureMeta = _test.FixtureMeta
FixtureRegistry = _test.FixtureRegistry

# Parametrize types from Rust
ParameterValue = _test.ParameterValue
ParameterSet = _test.ParameterSet
Parameter = _test.Parameter
ParametrizedTest = _test.ParametrizedTest

# Hooks types from Rust
HookType = _test.HookType
HookRegistry = _test.HookRegistry

# Import Python layer
from .decorators import test, profile, stress, security, fixture, parametrize
from .suite import (
    TestSuite, run_suite, run_suites,
    run_suite_with_coverage, run_suites_with_coverage,
)
from .benchmark import (
    benchmark,
    BenchmarkGroup,
    benchmark_decorator,
    ParametrizedBenchmark,
    # Discovery
    register_group,
    discover_benchmarks,
    run_benchmarks,
    clear_registry,
)
from .http_benchmark import HttpBenchmark
from .lazy_loader import lazy_load_test_suite, lazy_load_benchmark, lazy_load_test_module
from .profiler import ProfileRunner

__all__ = [
    # Enums
    "TestType",
    "TestStatus",
    "ReportFormat",
    # Core types
    "TestMeta",
    "TestResult",
    "TestSummary",
    "TestRunner",
    # Assertions
    "Expectation",
    "expect",
    # Reporter
    "Reporter",
    "TestReport",
    # Decorators
    "test",
    "profile",
    "stress",
    "security",
    "fixture",
    "parametrize",
    # Suite
    "TestSuite",
    "run_suite",
    "run_suites",
    "run_suite_with_coverage",
    "run_suites_with_coverage",
    # Benchmark
    "BenchmarkStats",
    "BenchmarkResult",
    "BenchmarkConfig",
    "benchmark",
    "BenchmarkGroup",
    "compare_benchmarks",
    "print_comparison_table",
    "benchmark_decorator",
    "ParametrizedBenchmark",
    # Benchmark Discovery
    "register_group",
    "discover_benchmarks",
    "run_benchmarks",
    "clear_registry",
    # Benchmark Report
    "BenchmarkEnvironment",
    "BenchmarkReportGroup",
    "BenchmarkReport",
    # Coverage
    "FileCoverage",
    "CoverageInfo",
    # Test Server
    "TestServer",
    "TestServerHandle",
    # HTTP Benchmark
    "HttpBenchmark",
    # Discovery
    "FileType",
    "FileInfo",
    "DiscoveryConfig",
    "TestRegistry",
    "BenchmarkRegistry",
    "DiscoveryStats",
    "discover_files",
    "filter_files_by_pattern",
    # Lazy Loading
    "lazy_load_test_suite",
    "lazy_load_benchmark",
    "lazy_load_test_module",
    # Profiler
    "ProfilePhase",
    "PhaseTiming",
    "PhaseBreakdown",
    "GilTestConfig",
    "GilContentionResult",
    "MemorySnapshot",
    "MemoryProfile",
    "FlamegraphData",
    "ProfileResult",
    "ProfileConfig",
    "ProfileRunner",
    "generate_flamegraph",
    # Fixtures
    "FixtureScope",
    "FixtureMeta",
    "FixtureRegistry",
    # Parametrize
    "ParameterValue",
    "ParameterSet",
    "Parameter",
    "ParametrizedTest",
    # Hooks
    "HookType",
    "HookRegistry",
]
