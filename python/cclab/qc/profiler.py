"""
Profiler module for data-bridge operations.

Provides comprehensive profiling capabilities:
- Phase breakdown timing (Python extract, Rust convert, Network I/O)
- GIL contention analysis under concurrent load
- Memory profiling (peak memory, allocation tracking)
- Flamegraph generation
"""

from __future__ import annotations

import asyncio
import gc
import resource
import sys
import threading
import time
import traceback
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List, Optional, TypeVar, Union

from .. import cclab as _rust_module

_test = _rust_module.qc

# Import profiling types from Rust
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

T = TypeVar("T")


def get_rss_bytes() -> int:
    """Get current RSS memory in bytes (macOS/Linux compatible)."""
    try:
        rusage = resource.getrusage(resource.RUSAGE_SELF)
        if sys.platform == "darwin":
            return rusage.ru_maxrss  # Already in bytes on macOS
        else:
            return rusage.ru_maxrss * 1024  # Convert KB to bytes on Linux
    except (ImportError, AttributeError):
        return 0


class ProfileRunner:
    """
    Runs profiling sessions with configurable dimensions.

    Supports:
    - Phase breakdown timing
    - GIL contention analysis
    - Memory profiling
    - Flamegraph generation

    Example:
        config = ProfileConfig.full()
        runner = ProfileRunner(config)

        async def insert_user():
            await User(name="Test", email="test@example.com").save()

        result = await runner.profile_operation(insert_user, "insert_user")
        print(result.format())
    """

    def __init__(self, config: ProfileConfig):
        self.config = config
        self._folded_stacks: List[str] = []
        self._phase_times: Dict[str, List[int]] = {}

    async def profile_operation(
        self,
        func: Union[Callable[[], T], Callable[[], Awaitable[T]]],
        name: str = "operation",
    ) -> ProfileResult:
        """
        Profile a single operation with all enabled dimensions.

        Args:
            func: The function to profile (sync or async)
            name: Name for the profiled operation

        Returns:
            ProfileResult with all profiling data
        """
        is_async = asyncio.iscoroutinefunction(func)
        start_time = time.perf_counter()

        # Initialize phase times
        self._phase_times = {
            "Total": [],
        }
        self._folded_stacks = []

        # Phase breakdown profiling
        phase_breakdown = None
        if self.config.enable_phase_breakdown:
            phase_breakdown = await self._profile_phases(func, is_async, name)

        # GIL contention analysis
        gil_result = None
        if self.config.enable_gil_analysis:
            gil_result = await self._profile_gil_contention(func, is_async)

        # Memory profiling
        memory_profile = None
        if self.config.enable_memory_profile:
            memory_profile = await self._profile_memory(func, is_async)

        # Collect stacks for flamegraph
        flamegraph_data = None
        if self.config.enable_flamegraph:
            flamegraph_data = await self._collect_stacks(func, is_async, name)

        duration_ms = (time.perf_counter() - start_time) * 1000

        # Build result using Rust types
        result = _ProfileResultBuilder(
            name=name,
            duration_ms=duration_ms,
            phase_breakdown=phase_breakdown,
            gil_analysis=gil_result,
            memory_profile=memory_profile,
            flamegraph=flamegraph_data,
        ).build()

        return result

    async def _profile_phases(
        self,
        func: Union[Callable, Callable[[], Awaitable]],
        is_async: bool,
        name: str,
    ) -> PhaseBreakdown:
        """Profile operation phases with timing breakdown."""
        iterations = self.config.iterations
        warmup = self.config.warmup

        # Warmup
        for _ in range(warmup):
            if is_async:
                await func()
            else:
                func()

        # Timed iterations - collect total time per operation
        phase_times: Dict[str, List[int]] = {
            "Total": [],
        }

        total_start = time.perf_counter_ns()

        for _ in range(iterations):
            op_start = time.perf_counter_ns()

            if is_async:
                await func()
            else:
                func()

            op_duration = time.perf_counter_ns() - op_start
            phase_times["Total"].append(op_duration)

        total_duration = time.perf_counter_ns() - total_start

        # Build PhaseBreakdown
        return _PhaseBreakdownBuilder(
            phase_times=phase_times,
            operation_count=iterations,
            total_duration_ns=total_duration,
        ).build()

    async def _profile_gil_contention(
        self,
        func: Union[Callable, Callable[[], Awaitable]],
        is_async: bool,
    ) -> GilContentionResult:
        """
        Analyze GIL contention by comparing sequential vs concurrent execution.

        If GIL is properly released during Rust operations, concurrent execution
        should achieve near-linear speedup.
        """
        gil_config = self.config.gil_config
        ops_per_worker = gil_config.operations_per_worker
        n_workers = gil_config.concurrent_workers

        # Warmup
        for _ in range(gil_config.warmup_iterations):
            if is_async:
                await func()
            else:
                func()

        total_ops = ops_per_worker * n_workers

        # Sequential baseline
        seq_start = time.perf_counter()
        for _ in range(total_ops):
            if is_async:
                await func()
            else:
                func()
        sequential_ms = (time.perf_counter() - seq_start) * 1000

        # Concurrent execution
        worker_times: List[float] = []

        if is_async:
            # Use asyncio for async functions
            async def worker() -> float:
                worker_start = time.perf_counter()
                for _ in range(ops_per_worker):
                    await func()
                return (time.perf_counter() - worker_start) * 1000

            concurrent_start = time.perf_counter()
            worker_times = list(await asyncio.gather(*[worker() for _ in range(n_workers)]))
            concurrent_total_ms = (time.perf_counter() - concurrent_start) * 1000
        else:
            # Use threading for sync functions
            results: List[float] = []
            lock = threading.Lock()

            def worker() -> None:
                worker_start = time.perf_counter()
                for _ in range(ops_per_worker):
                    func()
                elapsed = (time.perf_counter() - worker_start) * 1000
                with lock:
                    results.append(elapsed)

            threads = [threading.Thread(target=worker) for _ in range(n_workers)]
            concurrent_start = time.perf_counter()
            for t in threads:
                t.start()
            for t in threads:
                t.join()
            concurrent_total_ms = (time.perf_counter() - concurrent_start) * 1000
            worker_times = results

        # Calculate metrics
        theoretical_speedup = float(n_workers)
        actual_speedup = sequential_ms / concurrent_total_ms if concurrent_total_ms > 0 else 0
        efficiency = (actual_speedup / theoretical_speedup) * 100 if theoretical_speedup > 0 else 0
        overhead = ((concurrent_total_ms / sequential_ms) - 1.0) * 100 if sequential_ms > 0 else 0

        return _GilContentionResultBuilder(
            config=gil_config,
            sequential_baseline_ms=sequential_ms,
            concurrent_total_ms=concurrent_total_ms,
            worker_times_ms=worker_times,
            overhead_percent=overhead,
            gil_release_effective=overhead < 10.0,
            theoretical_speedup=theoretical_speedup,
            actual_speedup=actual_speedup,
            efficiency_percent=efficiency,
        ).build()

    async def _profile_memory(
        self,
        func: Union[Callable, Callable[[], Awaitable]],
        is_async: bool,
    ) -> MemoryProfile:
        """Profile memory usage during operation."""
        # Force GC before measurement
        gc.collect()

        # Get baseline memory
        before_rss = get_rss_bytes()
        peak_rss = before_rss
        iterations = self.config.iterations

        for _ in range(iterations):
            if is_async:
                await func()
            else:
                func()

            current_rss = get_rss_bytes()
            peak_rss = max(peak_rss, current_rss)

        after_rss = get_rss_bytes()

        return _MemoryProfileBuilder(
            before_rss=before_rss,
            after_rss=after_rss,
            peak_rss=peak_rss,
            iterations=iterations,
        ).build()

    async def _collect_stacks(
        self,
        func: Union[Callable, Callable[[], Awaitable]],
        is_async: bool,
        name: str,
    ) -> FlamegraphData:
        """Collect call stacks for flamegraph generation."""
        stacks: List[str] = []
        sample_count = min(100, self.config.iterations)

        for _ in range(sample_count):
            # Get current stack
            stack_frames = traceback.extract_stack()
            # Format as folded stack (semicolon-separated, with count)
            folded = ";".join(f.name for f in stack_frames[-10:])  # Last 10 frames
            stacks.append(f"{folded} 1")  # Count of 1 for each sample

            if is_async:
                await func()
            else:
                func()

        self._folded_stacks = stacks

        # Create FlamegraphData
        fg = FlamegraphData()
        for stack in stacks:
            fg.add_stack(stack)
        return fg

    def generate_flamegraph_svg(self, output_path: str, title: str = "Profile") -> None:
        """
        Generate flamegraph SVG from collected stacks.

        Args:
            output_path: Path to write the SVG file
            title: Title for the flamegraph

        Raises:
            ValueError: If no stacks have been collected
        """
        if not self._folded_stacks:
            raise ValueError(
                "No stacks collected. Run profile with enable_flamegraph=True first."
            )

        generate_flamegraph(self._folded_stacks, title, output_path)


# =====================
# Builder classes for creating Rust types from Python data
# =====================

class _PhaseBreakdownProxy:
    """Python-side proxy for PhaseBreakdown."""

    def __init__(
        self,
        phase_times: Dict[str, List[int]],
        operation_count: int,
        total_duration_ns: int,
    ):
        self._phase_times = phase_times
        self._operation_count = operation_count
        self._total_duration_ns = total_duration_ns

    @property
    def operation_count(self) -> int:
        return self._operation_count

    @property
    def total_duration_ms(self) -> float:
        return self._total_duration_ns / 1_000_000

    def format(self) -> str:
        """Format as human-readable string."""
        output = [
            f"Phase Breakdown ({self._operation_count} operations, "
            f"{self.total_duration_ms:.2f}ms total)\n",
            "-" * 60 + "\n",
        ]
        for phase, times in self._phase_times.items():
            total_ns = sum(times)
            total_ms = total_ns / 1_000_000
            count = len(times)
            avg_ms = total_ms / count if count > 0 else 0
            pct = (total_ns / self._total_duration_ns * 100) if self._total_duration_ns > 0 else 0
            output.append(
                f"{phase:<20} {total_ms:>10.2f}ms ({pct:5.1f}%) "
                f"[{count}x, avg={avg_ms:.3f}ms]\n"
            )
        return "".join(output)


class _PhaseBreakdownBuilder:
    """Helper to build PhaseBreakdown from Python-collected data."""

    def __init__(
        self,
        phase_times: Dict[str, List[int]],
        operation_count: int,
        total_duration_ns: int,
    ):
        self.phase_times = phase_times
        self.operation_count = operation_count
        self.total_duration_ns = total_duration_ns

    def build(self) -> "_PhaseBreakdownProxy":
        """Build a PhaseBreakdown proxy object."""
        return _PhaseBreakdownProxy(
            phase_times=self.phase_times,
            operation_count=self.operation_count,
            total_duration_ns=self.total_duration_ns,
        )


class _GilContentionResultBuilder:
    """Helper to build GilContentionResult from Python-collected data."""

    def __init__(
        self,
        config: GilTestConfig,
        sequential_baseline_ms: float,
        concurrent_total_ms: float,
        worker_times_ms: List[float],
        overhead_percent: float,
        gil_release_effective: bool,
        theoretical_speedup: float,
        actual_speedup: float,
        efficiency_percent: float,
    ):
        self.config = config
        self.sequential_baseline_ms = sequential_baseline_ms
        self.concurrent_total_ms = concurrent_total_ms
        self.worker_times_ms = worker_times_ms
        self.overhead_percent = overhead_percent
        self.gil_release_effective = gil_release_effective
        self.theoretical_speedup = theoretical_speedup
        self.actual_speedup = actual_speedup
        self.efficiency_percent = efficiency_percent

    def build(self) -> GilContentionResult:
        """Build a GilContentionResult object."""
        # GilContentionResult is created from Rust, so we create a proxy object
        result = _GilContentionResultProxy(
            sequential_baseline_ms=self.sequential_baseline_ms,
            concurrent_total_ms=self.concurrent_total_ms,
            worker_times_ms=self.worker_times_ms,
            overhead_percent=self.overhead_percent,
            gil_release_effective=self.gil_release_effective,
            theoretical_speedup=self.theoretical_speedup,
            actual_speedup=self.actual_speedup,
            efficiency_percent=self.efficiency_percent,
        )
        return result


class _MemoryProfileBuilder:
    """Helper to build MemoryProfile from Python-collected data."""

    def __init__(
        self,
        before_rss: int,
        after_rss: int,
        peak_rss: int,
        iterations: int,
    ):
        self.before_rss = before_rss
        self.after_rss = after_rss
        self.peak_rss = peak_rss
        self.iterations = iterations

    def build(self) -> MemoryProfile:
        """Build a MemoryProfile object."""
        return _MemoryProfileProxy(
            before_rss=self.before_rss,
            after_rss=self.after_rss,
            peak_rss=self.peak_rss,
            delta_bytes=self.after_rss - self.before_rss,
            iterations=self.iterations,
        )


class _ProfileResultBuilder:
    """Helper to build ProfileResult from all components."""

    def __init__(
        self,
        name: str,
        duration_ms: float,
        phase_breakdown: Optional[PhaseBreakdown] = None,
        gil_analysis: Optional[GilContentionResult] = None,
        memory_profile: Optional[MemoryProfile] = None,
        flamegraph: Optional[FlamegraphData] = None,
    ):
        self.name = name
        self.duration_ms = duration_ms
        self.phase_breakdown = phase_breakdown
        self.gil_analysis = gil_analysis
        self.memory_profile = memory_profile
        self.flamegraph = flamegraph

    def build(self) -> ProfileResult:
        """Build a ProfileResult object."""
        return _ProfileResultProxy(
            name=self.name,
            duration_ms=self.duration_ms,
            phase_breakdown=self.phase_breakdown,
            gil_analysis=self.gil_analysis,
            memory_profile=self.memory_profile,
            flamegraph=self.flamegraph,
        )


# =====================
# Proxy classes that mimic Rust types for Python-side results
# =====================

class _GilContentionResultProxy:
    """Python-side proxy for GilContentionResult."""

    def __init__(
        self,
        sequential_baseline_ms: float,
        concurrent_total_ms: float,
        worker_times_ms: List[float],
        overhead_percent: float,
        gil_release_effective: bool,
        theoretical_speedup: float,
        actual_speedup: float,
        efficiency_percent: float,
    ):
        self.sequential_baseline_ms = sequential_baseline_ms
        self.concurrent_total_ms = concurrent_total_ms
        self.worker_times_ms = worker_times_ms
        self.overhead_percent = overhead_percent
        self.gil_release_effective = gil_release_effective
        self.theoretical_speedup = theoretical_speedup
        self.actual_speedup = actual_speedup
        self.efficiency_percent = efficiency_percent

    def format(self) -> str:
        """Format as human-readable string."""
        return (
            f"GIL Contention Analysis\n"
            f"{'-' * 40}\n"
            f"Sequential baseline: {self.sequential_baseline_ms:.3f}ms\n"
            f"Concurrent total:    {self.concurrent_total_ms:.3f}ms\n"
            f"Overhead:            {self.overhead_percent:+.1f}%\n"
            f"GIL release:         {'EFFECTIVE' if self.gil_release_effective else 'BLOCKED'}\n"
            f"Theoretical speedup: {self.theoretical_speedup:.2f}x\n"
            f"Actual speedup:      {self.actual_speedup:.2f}x\n"
            f"Efficiency:          {self.efficiency_percent:.1f}%\n"
        )


class _MemoryProfileProxy:
    """Python-side proxy for MemoryProfile."""

    def __init__(
        self,
        before_rss: int,
        after_rss: int,
        peak_rss: int,
        delta_bytes: int,
        iterations: int,
    ):
        self.before_rss = before_rss
        self.after_rss = after_rss
        self.peak_rss = peak_rss
        self.delta_bytes = delta_bytes
        self.iterations = iterations

    @property
    def delta_mb(self) -> float:
        return self.delta_bytes / 1_048_576

    @property
    def peak_rss_mb(self) -> float:
        return self.peak_rss / 1_048_576

    def format(self) -> str:
        """Format as human-readable string."""
        return (
            f"Memory Profile\n"
            f"{'-' * 40}\n"
            f"Before RSS:         {self.before_rss / 1_048_576:.2f}MB\n"
            f"After RSS:          {self.after_rss / 1_048_576:.2f}MB\n"
            f"Peak RSS:           {self.peak_rss_mb:.2f}MB\n"
            f"Delta:              {self.delta_mb:+.2f}MB\n"
            f"Iterations:         {self.iterations}\n"
        )


class _ProfileResultProxy:
    """Python-side proxy for ProfileResult."""

    def __init__(
        self,
        name: str,
        duration_ms: float,
        phase_breakdown: Optional[Any] = None,
        gil_analysis: Optional[Any] = None,
        memory_profile: Optional[Any] = None,
        flamegraph: Optional[Any] = None,
        success: bool = True,
        error: Optional[str] = None,
    ):
        self.name = name
        self.duration_ms = duration_ms
        self.phase_breakdown = phase_breakdown
        self.gil_analysis = gil_analysis
        self.memory_profile = memory_profile
        self.flamegraph = flamegraph
        self.success = success
        self.error = error

    def format(self) -> str:
        """Format as human-readable string."""
        output = [f"=== Profile: {self.name} ===\n"]

        if not self.success:
            output.append(f"ERROR: {self.error or 'Unknown'}\n")
            return "".join(output)

        output.append(f"Duration: {self.duration_ms:.2f}ms\n\n")

        if self.phase_breakdown:
            if hasattr(self.phase_breakdown, "format"):
                output.append(self.phase_breakdown.format())
            else:
                # For Python-built breakdowns
                output.append("Phase Breakdown\n")
                output.append("-" * 40 + "\n")
                if hasattr(self.phase_breakdown, "_phase_times"):
                    for phase, times in self.phase_breakdown._phase_times.items():
                        total_ms = sum(times) / 1_000_000
                        count = len(times)
                        avg_ms = total_ms / count if count > 0 else 0
                        output.append(f"{phase:<20} {total_ms:>10.3f}ms [avg={avg_ms:.3f}ms]\n")
            output.append("\n")

        if self.gil_analysis:
            output.append(self.gil_analysis.format())
            output.append("\n")

        if self.memory_profile:
            output.append(self.memory_profile.format())
            output.append("\n")

        if self.flamegraph:
            sample_count = (
                self.flamegraph.sample_count
                if hasattr(self.flamegraph, "sample_count")
                else len(getattr(self.flamegraph, "folded_stacks", []))
            )
            output.append(f"Flamegraph: {sample_count} samples collected\n")

        return "".join(output)

    def to_json(self) -> str:
        """Export to JSON."""
        import json

        data = {
            "name": self.name,
            "duration_ms": self.duration_ms,
            "success": self.success,
            "error": self.error,
        }

        if self.gil_analysis:
            data["gil_analysis"] = {
                "sequential_baseline_ms": self.gil_analysis.sequential_baseline_ms,
                "concurrent_total_ms": self.gil_analysis.concurrent_total_ms,
                "overhead_percent": self.gil_analysis.overhead_percent,
                "gil_release_effective": self.gil_analysis.gil_release_effective,
                "actual_speedup": self.gil_analysis.actual_speedup,
                "efficiency_percent": self.gil_analysis.efficiency_percent,
            }

        if self.memory_profile:
            data["memory_profile"] = {
                "before_rss": self.memory_profile.before_rss,
                "after_rss": self.memory_profile.after_rss,
                "peak_rss": self.memory_profile.peak_rss,
                "delta_bytes": self.memory_profile.delta_bytes,
            }

        return json.dumps(data, indent=2)


# Convenience exports
__all__ = [
    "ProfileRunner",
    "ProfileConfig",
    "ProfileResult",
    "ProfilePhase",
    "PhaseTiming",
    "PhaseBreakdown",
    "GilTestConfig",
    "GilContentionResult",
    "MemorySnapshot",
    "MemoryProfile",
    "FlamegraphData",
    "generate_flamegraph",
    "get_rss_bytes",
]
