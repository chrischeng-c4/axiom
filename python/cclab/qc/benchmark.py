"""
Benchmark utilities for cclab.qc

Provides async-aware benchmark functions that collect timing data
and pass it to the Rust-backed BenchmarkResult for statistical analysis.

Features:
- benchmark() function for single benchmarks
- BenchmarkGroup for comparing multiple implementations
- benchmark_decorator for registering benchmarks
- discover_benchmarks() for file-based discovery
- run_benchmarks() for running discovered benchmarks
"""

from __future__ import annotations

import asyncio
import importlib.util
import sys
import time
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List, Optional, TypeVar, Union

# Import Rust types
from .. import cclab as _rust_module
_test = _rust_module.qc
BenchmarkResult = _test.BenchmarkResult
BenchmarkConfig = _test.BenchmarkConfig

T = TypeVar("T")

# Constants for auto-calibration
MIN_ITERATIONS = 10
MAX_ITERATIONS = 10_000
DEFAULT_TARGET_TIME_MS = 100.0  # Target total time for all iterations


async def _calibrate_iterations(
    func: Union[Callable[[], T], Callable[[], Awaitable[T]]],
    target_time_ms: float = DEFAULT_TARGET_TIME_MS,
    is_async: bool = False,
) -> int:
    """
    Run a few sample iterations to estimate optimal iteration count.

    Performs 3 sample runs to estimate time per operation, then calculates
    how many iterations are needed to reach target total time.

    Args:
        func: The function to benchmark.
        target_time_ms: Target total time for all iterations.
        is_async: Whether the function is async.

    Returns:
        Optimal number of iterations.
    """
    sample_times = []

    # Run 3 samples to estimate operation time
    for _ in range(3):
        start = time.perf_counter()
        if is_async:
            await func()
        else:
            # Run sync functions in a thread to avoid nested event loop conflicts
            await asyncio.to_thread(func)
        elapsed_ms = (time.perf_counter() - start) * 1000
        sample_times.append(elapsed_ms)

    # Use minimum sample time (least affected by system noise)
    sample_time_ms = min(sample_times)

    if sample_time_ms <= 0:
        return MIN_ITERATIONS

    # Calculate iterations to reach target time
    estimated_iters = int(target_time_ms / sample_time_ms)
    return max(MIN_ITERATIONS, min(MAX_ITERATIONS, estimated_iters))


async def benchmark(
    func: Union[Callable[[], T], Callable[[], Awaitable[T]]],
    *,
    name: Optional[str] = None,
    iterations: Optional[int] = None,
    rounds: int = 5,
    warmup: int = 3,
    auto: bool = False,
    target_time_ms: float = DEFAULT_TARGET_TIME_MS,
) -> BenchmarkResult:
    """
    Run a benchmark on a function.

    Executes the function multiple times and collects timing statistics.
    Works with both sync and async functions.

    Args:
        func: The function to benchmark (sync or async).
        name: Name for this benchmark (default: function name).
        iterations: Number of iterations per round. If None and auto=True,
            auto-calibrates. If None and auto=False, uses 20.
        rounds: Number of rounds to run.
        warmup: Number of warmup iterations before measuring.
        auto: If True, automatically calibrate iterations for optimal timing.
        target_time_ms: Target time per round when auto=True (default: 100ms).

    Returns:
        BenchmarkResult with timing statistics from Rust.

    Example:
        # Manual configuration
        result = await benchmark(my_func, iterations=100, rounds=5)

        # Auto-calibration (recommended)
        result = await benchmark(my_func, auto=True)
        print(result.format())
    """
    # Determine if async
    is_async = asyncio.iscoroutinefunction(func)

    # Get name
    benchmark_name = name or getattr(func, "__name__", "unknown")

    # Auto-calibrate iterations if needed
    if iterations is None:
        if auto:
            # Run calibration first
            iterations = await _calibrate_iterations(func, target_time_ms, is_async)
        else:
            iterations = 20  # Default

    # Warmup
    for _ in range(warmup):
        if is_async:
            await func()
        else:
            await asyncio.to_thread(func)

    # Collect times
    all_times: List[float] = []

    for _ in range(rounds):
        for _ in range(iterations):
            start = time.perf_counter()

            if is_async:
                await func()
            else:
                await asyncio.to_thread(func)

            elapsed_ms = (time.perf_counter() - start) * 1000
            all_times.append(elapsed_ms)

    # Create result using Rust-backed class for statistics
    return BenchmarkResult.from_times(
        benchmark_name,
        all_times,
        iterations,
        rounds,
        warmup,
    )


class BenchmarkGroup:
    """
    Group related benchmarks together for comparison.

    Example:
        group = BenchmarkGroup("HTTP GET")

        @group.add("data-bridge")
        async def databridge_get():
            return await db_client.get("/get")

        @group.add("httpx")
        async def httpx_get():
            return await httpx_client.get("/get")

        await group.run()
        print(group.report())
    """

    def __init__(self, name: str):
        self.name = name
        self.benchmarks: List[tuple[str, Callable]] = []
        self.results: List[BenchmarkResult] = []

    def add(self, name: str):
        """Decorator to add a benchmark to this group."""

        def decorator(func):
            self.benchmarks.append((name, func))
            return func

        return decorator

    async def run(
        self,
        iterations: Optional[int] = None,
        rounds: int = 5,
        warmup: int = 3,
        auto: bool = False,
        target_time_ms: float = DEFAULT_TARGET_TIME_MS,
    ) -> List[BenchmarkResult]:
        """
        Run all benchmarks in this group.

        Args:
            iterations: Number of iterations per round. If None and auto=True,
                auto-calibrates each benchmark separately.
            rounds: Number of rounds to run.
            warmup: Number of warmup iterations before measuring.
            auto: If True, automatically calibrate iterations for optimal timing.
            target_time_ms: Target time per round when auto=True (default: 100ms).

        Returns:
            List of BenchmarkResult objects.
        """
        self.results = []

        for name, func in self.benchmarks:
            result = await benchmark(
                func,
                name=name,
                iterations=iterations,
                rounds=rounds,
                warmup=warmup,
                auto=auto,
                target_time_ms=target_time_ms,
            )
            self.results.append(result)

        return self.results

    def report(self, baseline_name: Optional[str] = None) -> str:
        """Generate a comparison report for this group."""
        from cclab.qc import compare_benchmarks

        header = f"\n{self.name}\n"
        return header + compare_benchmarks(self.results, baseline_name)


class benchmark_decorator:
    """
    Decorator to mark a function as a benchmark.

    Can be used to decorate functions that should be run as benchmarks.
    The decorated function can then be run with .run() to execute the benchmark.

    Example:
        @benchmark_decorator(name="my_operation", auto=True)
        async def my_benchmark():
            await do_something()

        # Run the benchmark
        result = await my_benchmark.run()
        print(result.format())

        # Or use parametrize for multiple variations
        @benchmark_decorator.parametrize("size", [100, 1000, 10000])
        async def parametrized_benchmark(size: int):
            return sum(range(size))

        results = await parametrized_benchmark.run_all()
    """

    _registry: List["benchmark_decorator"] = []

    def __init__(
        self,
        name: Optional[str] = None,
        *,
        iterations: Optional[int] = None,
        rounds: int = 5,
        warmup: int = 3,
        auto: bool = True,
        target_time_ms: float = DEFAULT_TARGET_TIME_MS,
        group: Optional[str] = None,
    ):
        """
        Create a benchmark decorator.

        Args:
            name: Name for this benchmark (default: function name).
            iterations: Number of iterations per round.
            rounds: Number of rounds to run.
            warmup: Number of warmup iterations.
            auto: If True, auto-calibrate iterations.
            target_time_ms: Target time per round when auto=True.
            group: Optional group name for organizing benchmarks.
        """
        self.name = name
        self.iterations = iterations
        self.rounds = rounds
        self.warmup = warmup
        self.auto = auto
        self.target_time_ms = target_time_ms
        self.group = group
        self.func: Optional[Callable] = None
        self._parameters: List[tuple[str, List[Any]]] = []

    def __call__(self, func: Callable) -> "benchmark_decorator":
        """Decorate a function."""
        self.func = func
        if self.name is None:
            self.name = func.__name__
        benchmark_decorator._registry.append(self)
        return self

    async def run(self, **kwargs) -> BenchmarkResult:
        """
        Run this benchmark.

        Args:
            **kwargs: Additional arguments to pass to the function.

        Returns:
            BenchmarkResult with timing statistics.
        """
        if self.func is None:
            raise ValueError("No function to benchmark")

        # If kwargs provided, create a wrapper
        if kwargs:
            is_async = asyncio.iscoroutinefunction(self.func)
            if is_async:
                async def wrapper():
                    return await self.func(**kwargs)
            else:
                def wrapper():
                    return self.func(**kwargs)
            target = wrapper
        else:
            target = self.func

        return await benchmark(
            target,
            name=self.name,
            iterations=self.iterations,
            rounds=self.rounds,
            warmup=self.warmup,
            auto=self.auto,
            target_time_ms=self.target_time_ms,
        )

    @staticmethod
    def parametrize(param_name: str, values: List[Any]):
        """
        Create a parametrized benchmark decorator.

        Example:
            @benchmark_decorator.parametrize("size", [100, 1000, 10000])
            async def benchmark_sum(size: int):
                return sum(range(size))

            results = await benchmark_sum.run_all()
        """
        def decorator(func: Callable) -> "ParametrizedBenchmark":
            return ParametrizedBenchmark(func, param_name, values)
        return decorator

    @classmethod
    def get_all(cls) -> List["benchmark_decorator"]:
        """Get all registered benchmark decorators."""
        return cls._registry.copy()

    @classmethod
    async def run_all_registered(cls) -> List[BenchmarkResult]:
        """Run all registered benchmarks."""
        results = []
        for bm in cls._registry:
            result = await bm.run()
            results.append(result)
        return results


class ParametrizedBenchmark:
    """
    A benchmark that runs with multiple parameter values.
    """

    def __init__(self, func: Callable, param_name: str, values: List[Any]):
        self.func = func
        self.param_name = param_name
        self.values = values
        self.name = func.__name__
        self.iterations: Optional[int] = None
        self.rounds: int = 5
        self.warmup: int = 3
        self.auto: bool = True
        self.target_time_ms: float = DEFAULT_TARGET_TIME_MS

    async def run_all(self) -> List[BenchmarkResult]:
        """Run benchmark for all parameter values."""
        results = []
        is_async = asyncio.iscoroutinefunction(self.func)

        for value in self.values:
            # Create wrapper with parameter
            if is_async:
                async def wrapper(v=value):
                    return await self.func(**{self.param_name: v})
            else:
                def wrapper(v=value):
                    return self.func(**{self.param_name: v})

            result = await benchmark(
                wrapper,
                name=f"{self.name}[{self.param_name}={value}]",
                iterations=self.iterations,
                rounds=self.rounds,
                warmup=self.warmup,
                auto=self.auto,
                target_time_ms=self.target_time_ms,
            )
            results.append(result)

        return results


# =============================================================================
# Benchmark Discovery & Running
# =============================================================================

# Import Rust report types
BenchmarkReport = _test.BenchmarkReport
BenchmarkReportGroup = _test.BenchmarkReportGroup

# Global registry for BenchmarkGroups discovered from files
_group_registry: List[BenchmarkGroup] = []


def register_group(group: BenchmarkGroup) -> BenchmarkGroup:
    """Register a BenchmarkGroup for discovery."""
    _group_registry.append(group)
    return group


def get_registered_groups() -> List[BenchmarkGroup]:
    """Get all registered BenchmarkGroups."""
    return _group_registry.copy()


def clear_registry() -> None:
    """Clear all registered benchmarks and groups."""
    benchmark_decorator._registry.clear()
    _group_registry.clear()


def discover_benchmarks(
    path: Union[str, Path],
    pattern: str = "bench_*.py",
) -> Dict[str, Any]:
    """
    Discover benchmark files and import them.

    Finds all files matching the pattern, imports them to register
    benchmarks via decorators or register_group().

    Args:
        path: Directory to search for benchmark files.
        pattern: Glob pattern for benchmark files (default: bench_*.py).

    Returns:
        Dict with 'files', 'groups', 'decorators' counts.

    Example:
        from cclab.qc import discover_benchmarks, run_benchmarks

        discover_benchmarks("tests/benchmarks")
        await run_benchmarks()
    """
    path = Path(path)
    if not path.exists():
        raise FileNotFoundError(f"Benchmark path not found: {path}")

    # Clear previous registrations
    clear_registry()

    files_loaded = []

    # Enable relative imports by treating the directory as a package
    package_name = path.name
    parent_dir = str(path.parent)
    if parent_dir not in sys.path:
        sys.path.insert(0, parent_dir)

    # Register the package's __init__.py if it exists
    init_file = path / "__init__.py"
    if init_file.exists() and package_name not in sys.modules:
        init_spec = importlib.util.spec_from_file_location(
            package_name,
            init_file,
            submodule_search_locations=[str(path)],
        )
        if init_spec and init_spec.loader:
            init_mod = importlib.util.module_from_spec(init_spec)
            init_mod.__package__ = package_name
            sys.modules[package_name] = init_mod
            try:
                init_spec.loader.exec_module(init_mod)
            except Exception:
                pass

    for bench_file in sorted(path.glob(pattern)):
        if bench_file.name.startswith("_"):
            continue

        # Import as a submodule of the package for relative import support
        module_name = f"{package_name}.{bench_file.stem}"
        spec = importlib.util.spec_from_file_location(module_name, bench_file)
        if spec and spec.loader:
            module = importlib.util.module_from_spec(spec)
            module.__package__ = package_name
            sys.modules[module_name] = module
            try:
                spec.loader.exec_module(module)
                files_loaded.append(bench_file.name)
            except Exception as e:
                print(f"Warning: Failed to load {bench_file.name}: {e}")

    return {
        "files": files_loaded,
        "groups": len(_group_registry),
        "decorators": len(benchmark_decorator._registry),
    }


async def _cleanup_postgres_tables():
    """
    Clean up PostgreSQL benchmark tables between groups.

    Truncates all framework tables and runs VACUUM ANALYZE to remove dead tuples
    and update statistics, ensuring consistent benchmark results.
    """
    import os

    postgres_uri = os.environ.get(
        "POSTGRES_URI",
        "postgresql://postgres:postgres@localhost:5432/cclab_benchmark"
    )

    # Only cleanup if PostgreSQL is being used
    if not postgres_uri.startswith("postgresql"):
        return

    try:
        import asyncpg

        # Parse URI to get connection parameters
        import urllib.parse
        parsed = urllib.parse.urlparse(postgres_uri)

        conn = await asyncpg.connect(
            host=parsed.hostname or "localhost",
            port=parsed.port or 5432,
            user=parsed.username or "postgres",
            password=parsed.password or "postgres",
            database=parsed.path.lstrip("/") or "cclab_benchmark",
        )

        try:
            # Truncate all benchmark tables to remove bloat
            await conn.execute("""
                TRUNCATE TABLE bench_db_users,
                             bench_asyncpg_users,
                             bench_psycopg2_users,
                             bench_sa_users
                RESTART IDENTITY CASCADE;
            """)

            # VACUUM ANALYZE to reclaim space and update statistics
            # Note: VACUUM cannot run inside a transaction block, but asyncpg runs each
            # statement in auto-commit mode by default
            await conn.execute("VACUUM ANALYZE bench_db_users;")
            await conn.execute("VACUUM ANALYZE bench_asyncpg_users;")
            await conn.execute("VACUUM ANALYZE bench_psycopg2_users;")
            await conn.execute("VACUUM ANALYZE bench_sa_users;")

        finally:
            await conn.close()

    except ImportError:
        # asyncpg not available, skip cleanup
        pass
    except Exception as e:
        # Cleanup failure shouldn't stop benchmarks
        print(f"Warning: PostgreSQL table cleanup failed: {e}")


async def run_benchmarks(
    *,
    auto: bool = True,
    rounds: int = 5,
    warmup: int = 3,
    baseline_name: Optional[str] = None,
    title: str = "Benchmark Results",
    description: Optional[str] = None,
) -> BenchmarkReport:
    """
    Run all discovered benchmarks and return a report.

    Runs both registered BenchmarkGroups and individual benchmark_decorators.

    Args:
        auto: Auto-calibrate iterations.
        rounds: Number of rounds per benchmark.
        warmup: Warmup iterations.
        baseline_name: Name of baseline for comparison reports.
        title: Report title.
        description: Report description.

    Returns:
        BenchmarkReport (Rust-backed) with all results.

    Example:
        from cclab.qc import discover_benchmarks, run_benchmarks

        discover_benchmarks("tests/benchmarks")
        report = await run_benchmarks(baseline_name="Beanie")
        print(report.to_console())
        report.save("benchmark_report", format="markdown")
    """
    report = BenchmarkReport(title=title, description=description)

    # Run BenchmarkGroups
    for group in _group_registry:
        print(f"\nRunning: {group.name}")

        # Clean up PostgreSQL tables before each group to prevent bloat
        await _cleanup_postgres_tables()

        await group.run(auto=auto, rounds=rounds, warmup=warmup)

        # Create Rust report group
        report_group = BenchmarkReportGroup(name=group.name, baseline=baseline_name)
        for result in group.results:
            report_group.add_result(result)
        report.add_group(report_group)

    # Run individual decorators (not in groups)
    ungrouped = [b for b in benchmark_decorator._registry if not b.group]
    if ungrouped:
        print("\nRunning: Individual Benchmarks")
        report_group = BenchmarkReportGroup(name="Individual", baseline=baseline_name)

        for bm in ungrouped:
            result = await bm.run()
            report_group.add_result(result)

        report.add_group(report_group)

    return report
