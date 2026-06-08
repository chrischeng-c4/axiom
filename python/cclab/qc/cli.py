"""
dbtest CLI - Unified test and benchmark runner with auto-discovery.

This module provides a command-line interface for running tests and benchmarks
with automatic file discovery powered by the Rust engine.
"""

from __future__ import annotations

import argparse
import asyncio
import os
import sys
from pathlib import Path
from typing import List, Optional

from .lazy_loader import lazy_load_test_suite, lazy_load_benchmark
from . import (
    DiscoveryConfig,
    discover_files,
    FileType,
    TestRunner,
    Reporter,
    ReportFormat,
    run_benchmarks,
    clear_registry,
)
from .profiler import (
    ProfileRunner,
    ProfileConfig,
    GilTestConfig,
    generate_flamegraph,
)


async def ensure_mongodb_initialized(verbose: bool = False) -> None:
    """
    Ensure MongoDB is initialized before running benchmarks or integration tests.
    
    Uses environment variables for connection string or a default value.
    Attempts to use benchmark_setup if available to initialize multiple frameworks (e.g., Beanie).
    """
    from cclab.mongo import init, is_connected
    
    if is_connected():
        return
        
    # Try to use benchmark_setup if it exists in the discovery path
    # This is useful for cross-framework benchmarks that need multiple inits
    try:
        # Check if we are running in the mongo benchmarks directory
        # This is a bit of a hack but common for these benchmarks
        import importlib
        try:
            setup_mod = importlib.import_module("tests.mongo.benchmarks.benchmark_setup")
            if verbose:
                print(f"🔌 Using benchmark_setup for multi-framework initialization")
            await setup_mod.async_ensure_setup()
            return
        except (ImportError, AttributeError):
            pass
    except Exception as e:
        if verbose:
            print(f"⚠️  Could not use benchmark_setup: {e}")

    # Fallback to standard data-bridge initialization
    uri = os.environ.get("MONGODB_BENCHMARK_URI") or os.environ.get("MONGODB_URI")
    if not uri:
        uri = "mongodb://localhost:27017/data-bridge"
        
    if verbose:
        print(f"🔌 Initializing data-bridge MongoDB connection: {uri}")
    else:
        print(f"🔌 Initializing MongoDB connection...")
        
    try:
        await init(uri)
    except Exception as e:
        print(f"❌ Failed to initialize MongoDB: {e}")
        raise


class CLIConfig:
    """Configuration for CLI execution."""

    def __init__(self):
        self.root_path: str = "python/tests/"
        self.patterns: List[str] = ["test_*.py", "bench_*.py"]
        self.exclusions: List[str] = ["__pycache__", ".git", ".venv", "node_modules"]
        self.max_depth: int = 10
        self.test_type: Optional[str] = None  # unit, integration, all
        self.run_benchmarks_flag: bool = False
        self.run_tests_flag: bool = True
        self.run_profile_flag: bool = False
        self.pattern_filter: Optional[str] = None
        self.tags: List[str] = []
        self.verbose: bool = False
        self.fail_fast: bool = False
        self.format: str = "console"  # console, json, markdown
        self.output_file: Optional[str] = None
        self.parsed_args: Optional[argparse.Namespace] = None  # Store raw parsed args for profile


def create_discovery_config(cli_config: CLIConfig) -> DiscoveryConfig:
    """
    Create a DiscoveryConfig from CLIConfig.

    Args:
        cli_config: CLI configuration

    Returns:
        DiscoveryConfig for Rust discovery engine
    """
    return DiscoveryConfig(
        root_path=cli_config.root_path,
        patterns=cli_config.patterns,
        exclusions=cli_config.exclusions,
        max_depth=cli_config.max_depth,
    )


async def run_tests_only(cli_config: CLIConfig) -> int:
    """
    Run only tests (no benchmarks).

    Args:
        cli_config: CLI configuration

    Returns:
        Exit code (0 = success, 1 = failures, 2 = errors)
    """
    # Create discovery config
    patterns = ["test_*.py"]
    if cli_config.pattern_filter:
        patterns = [cli_config.pattern_filter]

    discovery_config = DiscoveryConfig(
        root_path=cli_config.root_path,
        patterns=patterns,
        exclusions=cli_config.exclusions,
        max_depth=cli_config.max_depth,
    )

    # Discover test files
    print(f"🔍 Discovering test files in {cli_config.root_path}...")
    files = discover_files(discovery_config)

    # Filter by file type
    test_files = [f for f in files if f.file_type == FileType.Test]

    if not test_files:
        print("❌ No test files found")
        return 1

    print(f"✅ Found {len(test_files)} test file(s)")

    # Ensure MongoDB is initialized for integration tests
    if cli_config.test_type == "integration":
        await ensure_mongodb_initialized(cli_config.verbose)

    # Load and run test suites
    total_passed = 0
    total_failed = 0
    total_errors = 0

    for file_info in test_files:
        file_path = Path(file_info.path)

        if cli_config.verbose:
            print(f"\n📄 Loading: {file_info.module_name}")

        try:
            # Lazy load test suites from file
            suites = lazy_load_test_suite(file_path)

            for suite_class in suites:
                if cli_config.verbose:
                    print(f"  🧪 Running: {suite_class.__name__}")

                # Create suite instance and run it
                suite = suite_class()
                runner = TestRunner()
                report = await suite.run(runner=runner, verbose=cli_config.verbose)

                # Aggregate results from summary
                summary = report.summary
                total_passed += summary.passed
                total_failed += summary.failed
                total_errors += summary.errors

                # Fail fast if requested
                if cli_config.fail_fast and (summary.failed > 0 or summary.errors > 0):
                    print(f"\n❌ Stopping due to --fail-fast")
                    break

        except Exception as e:
            print(f"❌ Error loading {file_path}: {e}")
            total_errors += 1
            if cli_config.fail_fast:
                break

        if cli_config.fail_fast and (total_failed > 0 or total_errors > 0):
            break

    # Print summary
    print("\n" + "=" * 60)
    print("TEST SUMMARY")
    print("=" * 60)
    print(f"✅ Passed:  {total_passed}")
    print(f"❌ Failed:  {total_failed}")
    print(f"⚠️  Errors:  {total_errors}")
    print("=" * 60)

    # Return appropriate exit code
    if total_errors > 0:
        return 2
    elif total_failed > 0:
        return 1
    else:
        return 0


async def run_benchmarks_only(cli_config: CLIConfig) -> int:
    """
    Run only benchmarks (no tests).

    Args:
        cli_config: CLI configuration

    Returns:
        Exit code (0 = success, 1 = failures)
    """
    # Clear benchmark registry before discovery
    clear_registry()

    # Create discovery config
    patterns = ["bench_*.py"]
    if cli_config.pattern_filter:
        patterns = [cli_config.pattern_filter]

    discovery_config = DiscoveryConfig(
        root_path=cli_config.root_path,
        patterns=patterns,
        exclusions=cli_config.exclusions,
        max_depth=cli_config.max_depth,
    )

    # Discover benchmark files
    print(f"🔍 Discovering benchmark files in {cli_config.root_path}...")
    files = discover_files(discovery_config)

    # Filter by file type
    bench_files = [f for f in files if f.file_type == FileType.Benchmark]

    if not bench_files:
        print("❌ No benchmark files found")
        return 1

    print(f"✅ Found {len(bench_files)} benchmark file(s)")

    # Ensure MongoDB is initialized BEFORE loading benchmark files
    # This is critical because benchmark files may import modules that require MongoDB
    await ensure_mongodb_initialized(cli_config.verbose)

    # Load benchmark groups
    all_groups = []
    for file_info in bench_files:
        file_path = Path(file_info.path)

        if cli_config.verbose:
            print(f"\n📄 Loading: {file_info.module_name}")

        try:
            # Lazy load benchmark groups from file
            groups = lazy_load_benchmark(file_path)
            all_groups.extend(groups)

            if cli_config.verbose:
                for group in groups:
                    print(f"  📊 Group: {group.name} ({len(group.benchmarks)} benchmarks)")

        except Exception as e:
            print(f"❌ Error loading {file_path}: {e}")

    if not all_groups:
        print("❌ No benchmark groups found")
        return 1

    # Run all benchmarks
    print(f"\n🏃 Running {len(all_groups)} benchmark group(s)...")

    try:
        # Use existing run_benchmarks function
        # This will run all registered groups
        report = await run_benchmarks()

        # Handle report output
        output_path = cli_config.output_file

        if output_path:
            # User explicitly specified output file - save to file
            print(f"\n💾 Saving report to {output_path}...")
            report.save(output_path, format=cli_config.format)
        else:
            # No output file specified - print to console
            if cli_config.format == "json":
                output = report.to_json()
            elif cli_config.format == "markdown":
                output = report.to_markdown()
            else:  # console (default)
                output = report.to_console()

            print(f"\n{output}")

        if cli_config.verbose:
            print(f"\n📊 Benchmark Report:")
            print(f"  Total groups: {len(all_groups)}")
            if output_path:
                print(f"  Output: {output_path}")
            else:
                print(f"  Output: console")

        print("\n✅ Benchmarks completed")
        return 0

    except Exception as e:
        print(f"❌ Benchmark execution failed: {e}")
        if cli_config.verbose:
            import traceback
            traceback.print_exc()
        return 1


async def run_all(cli_config: CLIConfig) -> int:
    """
    Run both tests and benchmarks.

    Args:
        cli_config: CLI configuration

    Returns:
        Exit code (0 = success, 1 = failures)
    """
    print("=" * 60)
    print("RUNNING TESTS")
    print("=" * 60)

    test_exit_code = await run_tests_only(cli_config)

    print("\n" + "=" * 60)
    print("RUNNING BENCHMARKS")
    print("=" * 60)

    bench_exit_code = await run_benchmarks_only(cli_config)

    # Return worst exit code
    return max(test_exit_code, bench_exit_code)


async def run_profile(cli_config: CLIConfig) -> int:
    """
    Run profiling on a specific operation.

    Args:
        cli_config: CLI configuration with profile settings

    Returns:
        Exit code (0 = success, 1 = failures)
    """
    from cclab.mongo import Document, init, close

    # Get profile-specific settings from parsed args
    parsed = cli_config.parsed_args
    target = parsed.target
    enable_all = getattr(parsed, "all", False)
    enable_phases = getattr(parsed, "phases", True) or enable_all
    enable_gil = getattr(parsed, "gil", False) or enable_all
    enable_memory = getattr(parsed, "memory", False) or enable_all
    enable_flamegraph = getattr(parsed, "flamegraph", False) or enable_all
    iterations = getattr(parsed, "iterations", 100)
    warmup = getattr(parsed, "warmup", 10)
    workers = getattr(parsed, "workers", 4)
    output_dir = getattr(parsed, "output", None)

    # Build config
    if enable_all:
        config = ProfileConfig.full()
    else:
        config = ProfileConfig(
            enable_phase_breakdown=enable_phases,
            enable_gil_analysis=enable_gil,
            enable_memory_profile=enable_memory,
            enable_flamegraph=enable_flamegraph,
            iterations=iterations,
            warmup=warmup,
            output_dir=output_dir,
        )

    if enable_gil:
        gil_config = GilTestConfig(
            concurrent_workers=workers,
            operations_per_worker=iterations // workers,
        )
        config = config.with_gil_config(gil_config)

    # Initialize MongoDB
    uri = os.environ.get("MONGODB_URI", "mongodb://localhost:27017/profile_db")
    if cli_config.verbose:
        print(f"Connecting to MongoDB: {uri}")
    else:
        print(f"Connecting to MongoDB...")

    try:
        await init(uri)
    except Exception as e:
        print(f"Failed to connect to MongoDB: {e}")
        return 1

    # Create test document class
    class ProfileDoc(Document):
        name: str
        value: int

        class Settings:
            name = "profile_test"

    # Create output directory if needed
    if output_dir:
        Path(output_dir).mkdir(parents=True, exist_ok=True)

    runner = ProfileRunner(config)

    try:
        print(f"\nProfiling: {target}")
        print("=" * 60)

        # Profile based on target
        if target == "insert_one":
            async def op():
                await ProfileDoc(name="test", value=42).save()

            result = await runner.profile_operation(op, "insert_one")

        elif target == "find_one":
            # Seed data first
            await ProfileDoc(name="find_target", value=100).save()

            async def op():
                await ProfileDoc.find_one(ProfileDoc.value == 100)

            result = await runner.profile_operation(op, "find_one")

        elif target == "find_many":
            # Seed data first
            for i in range(100):
                await ProfileDoc(name=f"bulk_{i}", value=i).save()

            async def op():
                await ProfileDoc.find(ProfileDoc.value >= 0).to_list()

            result = await runner.profile_operation(op, "find_many")

        elif target == "bulk_insert":
            async def op():
                docs = [ProfileDoc(name=f"bulk_{i}", value=i) for i in range(100)]
                await ProfileDoc.insert_many(docs)

            result = await runner.profile_operation(op, "bulk_insert")

        elif target == "update_one":
            # Seed data first
            await ProfileDoc(name="update_target", value=100).save()

            async def op():
                doc = await ProfileDoc.find_one(ProfileDoc.name == "update_target")
                if doc:
                    doc.value = doc.value + 1 if doc.value < 1000 else 100
                    await doc.save()

            result = await runner.profile_operation(op, "update_one")

        elif target == "delete_one":
            async def op():
                # Insert then delete
                doc = ProfileDoc(name="delete_target", value=999)
                await doc.save()
                await doc.delete()

            result = await runner.profile_operation(op, "delete_one")

        else:
            print(f"Unknown target: {target}")
            return 1

        # Print results
        print(result.format())

        # Save JSON if output directory specified
        if output_dir:
            json_path = Path(output_dir) / f"{target}_profile.json"
            with open(json_path, "w") as f:
                f.write(result.to_json())
            print(f"Saved JSON: {json_path}")

        # Generate flamegraph if enabled
        if enable_flamegraph and output_dir:
            try:
                svg_path = str(Path(output_dir) / f"{target}_flamegraph.svg")
                runner.generate_flamegraph_svg(svg_path, f"Profile: {target}")
                print(f"Saved flamegraph: {svg_path}")
            except Exception as e:
                print(f"Failed to generate flamegraph: {e}")

        print("\nProfile completed successfully")
        return 0

    except Exception as e:
        print(f"Profile failed: {e}")
        if cli_config.verbose:
            import traceback
            traceback.print_exc()
        return 1

    finally:
        # Cleanup - ignore errors during cleanup as we're exiting anyway
        try:
            await ProfileDoc.delete_many({})
        except Exception:  # nosec B110 - cleanup during shutdown can fail safely
            pass
        await close()


def parse_args(args: Optional[List[str]] = None) -> CLIConfig:
    """
    Parse command-line arguments.

    Args:
        args: Command-line arguments (defaults to sys.argv[1:])

    Returns:
        CLIConfig with parsed settings
    """
    parser = argparse.ArgumentParser(
        prog="dbtest",
        description="Unified test and benchmark runner with auto-discovery",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  dbtest                    # Run all tests and benchmarks
  dbtest unit               # Run unit tests only
  dbtest integration        # Run integration tests only
  dbtest bench              # Run benchmarks only
  dbtest --pattern "*crud*" # Run tests matching pattern
  dbtest --verbose          # Verbose output
  dbtest --fail-fast        # Stop on first failure
        """,
    )

    # Subcommands
    subparsers = parser.add_subparsers(dest="command", help="Command to run")

    # dbtest unit
    unit_parser = subparsers.add_parser("unit", help="Run unit tests only")
    unit_parser.add_argument("--pattern", help="File pattern to match")
    unit_parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    unit_parser.add_argument("--fail-fast", action="store_true", help="Stop on first failure")

    # dbtest integration
    integration_parser = subparsers.add_parser("integration", help="Run integration tests only")
    integration_parser.add_argument("--pattern", help="File pattern to match")
    integration_parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    integration_parser.add_argument("--fail-fast", action="store_true", help="Stop on first failure")

    # dbtest bench
    bench_parser = subparsers.add_parser("bench", help="Run benchmarks only")
    bench_parser.add_argument("--pattern", help="File pattern to match")
    bench_parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")

    # dbtest profile
    profile_parser = subparsers.add_parser(
        "profile",
        help="Profile data-bridge operations",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  dbtest profile insert_one                    # Profile insert_one with phase breakdown
  dbtest profile find_one --iterations 200     # More iterations for statistical significance
  dbtest profile bulk_insert --all             # Enable all profiling dimensions
  dbtest profile insert_one --flamegraph       # Generate flamegraph SVG
  dbtest profile find_one --gil                # Analyze GIL contention
  dbtest profile bulk_insert -o ./profiles     # Output to directory
        """,
    )
    profile_parser.add_argument(
        "target",
        choices=["insert_one", "find_one", "find_many", "bulk_insert", "update_one", "delete_one"],
        help="Operation to profile",
    )
    profile_parser.add_argument(
        "--phases", "-p",
        action="store_true",
        default=True,
        help="Enable phase breakdown (default: enabled)",
    )
    profile_parser.add_argument(
        "--gil", "-g",
        action="store_true",
        help="Enable GIL contention analysis",
    )
    profile_parser.add_argument(
        "--memory", "-m",
        action="store_true",
        help="Enable memory profiling",
    )
    profile_parser.add_argument(
        "--flamegraph", "-f",
        action="store_true",
        help="Generate flamegraph SVG",
    )
    profile_parser.add_argument(
        "--all", "-a",
        action="store_true",
        help="Enable all profiling dimensions",
    )
    profile_parser.add_argument(
        "--iterations", "-i",
        type=int,
        default=100,
        help="Number of profiling iterations (default: 100)",
    )
    profile_parser.add_argument(
        "--warmup", "-w",
        type=int,
        default=10,
        help="Warmup iterations (default: 10)",
    )
    profile_parser.add_argument(
        "--workers",
        type=int,
        default=4,
        help="Concurrent workers for GIL analysis (default: 4)",
    )
    profile_parser.add_argument(
        "--output", "-o",
        help="Output directory for profile results",
    )
    profile_parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Verbose output",
    )

    # Global options (for main command without subcommand)
    parser.add_argument("--root", default="python/tests/", help="Root directory to search (default: python/tests/)")
    parser.add_argument("--pattern", help="File pattern to match (e.g., test_*crud*.py)")
    parser.add_argument("--tags", nargs="+", help="Filter tests by tags")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    parser.add_argument("--fail-fast", action="store_true", help="Stop on first failure")
    parser.add_argument("--format", choices=["console", "json", "markdown"], default="console",
                        help="Output format (default: console)")
    parser.add_argument("--output", "-o", help="Output file path (if not specified, prints to console)")

    parsed = parser.parse_args(args)

    # Build CLIConfig
    config = CLIConfig()
    config.root_path = parsed.root
    config.verbose = parsed.verbose
    config.fail_fast = parsed.fail_fast
    config.format = parsed.format
    config.output_file = parsed.output

    if hasattr(parsed, "tags") and parsed.tags:
        config.tags = parsed.tags

    if parsed.pattern:
        config.pattern_filter = parsed.pattern

    # Handle subcommands
    if parsed.command == "unit":
        config.test_type = "unit"
        config.run_tests_flag = True
        config.run_benchmarks_flag = False
        config.patterns = ["test_*.py"]

    elif parsed.command == "integration":
        config.test_type = "integration"
        config.run_tests_flag = True
        config.run_benchmarks_flag = False
        config.patterns = ["test_*.py"]

    elif parsed.command == "bench":
        config.run_tests_flag = False
        config.run_benchmarks_flag = True
        config.patterns = ["bench_*.py"]

    elif parsed.command == "profile":
        config.run_tests_flag = False
        config.run_benchmarks_flag = False
        config.run_profile_flag = True
        config.parsed_args = parsed

    else:
        # No subcommand: run all
        config.run_tests_flag = True
        config.run_benchmarks_flag = True

    return config


def main(args: Optional[List[str]] = None) -> int:
    """
    Main entry point for dbtest CLI.

    Args:
        args: Command-line arguments (defaults to sys.argv[1:])

    Returns:
        Exit code (0 = success, 1 = failures, 2 = errors)
    """
    # Add python/ directory to path for test imports (tests.* modules)
    python_dir = Path(__file__).parent.parent.parent  # Navigate to python/
    if str(python_dir) not in sys.path:
        sys.path.insert(0, str(python_dir))

    config = parse_args(args)

    # Print banner
    print("=" * 60)
    print("dbtest - Data Bridge Test & Benchmark Runner")
    print("=" * 60)
    print()

    # Run appropriate command
    try:
        if config.run_profile_flag:
            # Profile only
            exit_code = asyncio.run(run_profile(config))

        elif config.run_benchmarks_flag and not config.run_tests_flag:
            # Benchmarks only
            exit_code = asyncio.run(run_benchmarks_only(config))

        elif config.run_tests_flag and not config.run_benchmarks_flag:
            # Tests only
            exit_code = asyncio.run(run_tests_only(config))

        else:
            # Both tests and benchmarks
            exit_code = asyncio.run(run_all(config))

        return exit_code

    except KeyboardInterrupt:
        print("\n\n⚠️  Interrupted by user")
        return 130  # Standard exit code for SIGINT

    except Exception as e:
        print(f"\n❌ Fatal error: {e}")
        if config.verbose:
            import traceback
            traceback.print_exc()
        return 2


if __name__ == "__main__":
    sys.exit(main())
