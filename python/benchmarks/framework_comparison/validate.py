#!/usr/bin/env python3
"""
Quick validation script for framework comparison benchmarks.

This script performs basic sanity checks to ensure the benchmark
infrastructure is working correctly before running the full benchmark.

Usage:
    python benchmarks/framework_comparison/validate.py
"""

import asyncio
import sys
from pathlib import Path

# Add project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))


def check_pytest():
    """Check if pytest is available."""
    try:
        import pytest
        print(f"✓ pytest {pytest.__version__} installed")
        return True
    except ImportError:
        print("✗ pytest not installed")
        print("  Install with: pip install pytest")
        return False


def check_psutil():
    """Check if psutil is available (optional)."""
    try:
        import psutil
        print(f"✓ psutil {psutil.__version__} installed (memory tracking enabled)")
        return True
    except ImportError:
        print("⚠ psutil not installed (memory tracking disabled)")
        print("  Install with: pip install psutil")
        return False


def check_cclab_test():
    """Check if data-bridge-test is available."""
    try:
        from cclab.probe import (
            TestSuite, test, expect, benchmark, BenchmarkGroup
        )
        print("✓ data-bridge-test available")
        return True
    except ImportError as e:
        print(f"✗ data-bridge-test not available: {e}")
        print("  Build with: maturin develop --release")
        return False


def check_sample_tests():
    """Check if sample tests file exists."""
    sample_path = Path(__file__).parent / "sample_tests.py"
    if sample_path.exists():
        print(f"✓ Sample tests found: {sample_path}")
        return True
    else:
        print(f"✗ Sample tests not found: {sample_path}")
        return False


async def test_pytest_execution():
    """Test pytest can run sample tests."""
    import subprocess
    sample_path = Path(__file__).parent / "sample_tests.py"

    try:
        result = subprocess.run(
            [sys.executable, "-m", "pytest", str(sample_path), "-v", "--tb=short"],
            capture_output=True,
            text=True,
            timeout=30,
        )

        if result.returncode == 0:
            # Count passed tests
            passed = result.stdout.count(" PASSED")
            print(f"✓ pytest execution successful ({passed} tests passed)")
            return True
        else:
            print(f"✗ pytest execution failed")
            print(f"  stdout: {result.stdout[:200]}")
            print(f"  stderr: {result.stderr[:200]}")
            return False
    except subprocess.TimeoutExpired:
        print("✗ pytest execution timed out")
        return False
    except Exception as e:
        print(f"✗ pytest execution error: {e}")
        return False


async def test_cclab_test_execution():
    """Test data-bridge-test can run basic tests."""
    try:
        from cclab.probe import TestSuite, test, expect, TestRunner

        class ValidationSuite(TestSuite):
            @test
            async def test_basic(self):
                expect(1 + 1).to_equal(2)

            @test
            async def test_string(self):
                expect("hello".upper()).to_equal("HELLO")

            @test
            async def test_list(self):
                expect(len([1, 2, 3])).to_equal(3)

        # Use suite.run() directly since we're already in async context
        runner = TestRunner()
        suite = ValidationSuite()
        report = await suite.run(runner=runner, verbose=False)

        if report.summary.passed == 3 and report.summary.failed == 0:
            print(f"✓ data-bridge-test execution successful ({report.summary.passed} tests passed)")
            return True
        else:
            print(f"✗ data-bridge-test execution failed")
            print(f"  Passed: {report.summary.passed}, Failed: {report.summary.failed}")
            return False
    except Exception as e:
        print(f"✗ data-bridge-test execution error: {e}")
        return False


async def test_benchmark_infrastructure():
    """Test basic benchmark functionality."""
    try:
        from cclab.probe import benchmark

        async def simple_operation():
            return sum(range(100))

        result = await benchmark(
            simple_operation,
            name="validation",
            iterations=10,
            rounds=3,
            warmup=2,
        )

        if result.stats.mean_ms > 0:
            print(f"✓ Benchmark infrastructure working (mean: {result.stats.mean_ms:.3f}ms)")
            return True
        else:
            print("✗ Benchmark infrastructure produced invalid results")
            return False
    except Exception as e:
        print(f"✗ Benchmark infrastructure error: {e}")
        return False


async def main():
    """Run all validation checks."""
    print("\n" + "=" * 70)
    print("Framework Comparison Benchmark Validation")
    print("=" * 70)
    print()

    checks = []

    # Dependency checks
    print("Checking dependencies...")
    print("-" * 70)
    checks.append(check_pytest())
    checks.append(check_psutil())
    checks.append(check_cclab_test())
    checks.append(check_sample_tests())
    print()

    # Execution checks
    print("Testing execution...")
    print("-" * 70)
    checks.append(await test_pytest_execution())
    checks.append(await test_cclab_test_execution())
    print()

    # Infrastructure checks
    print("Testing infrastructure...")
    print("-" * 70)
    checks.append(await test_benchmark_infrastructure())
    print()

    # Summary
    print("=" * 70)
    required_checks = [checks[0], checks[2], checks[3], checks[4], checks[5], checks[6]]
    all_required_passed = all(required_checks)

    if all_required_passed:
        print("✓ All required checks passed!")
        print()
        print("You can now run the full benchmark:")
        print("  python benchmarks/framework_comparison/pytest_vs_cclab_test.py")
        return 0
    else:
        print("✗ Some required checks failed")
        print()
        print("Please fix the issues above before running the benchmark.")
        return 1


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
