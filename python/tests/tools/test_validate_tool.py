"""Tests for migration validation tool."""

import tempfile
from pathlib import Path

from cclab.probe import TestSuite, test, expect


# Import the validation tool
import sys
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "tools"))
from validate_migration import TestStats, ValidationResult, ValidationReport


class TestTestStats(TestSuite):
    """Test TestStats dataclass."""

    @test
    def test_create_empty_stats(self):
        """Test creating empty stats."""
        stats = TestStats()
        expect(stats.total).to_equal(0)
        expect(stats.passed).to_equal(0)
        expect(stats.failed).to_equal(0)
        expect(stats.skipped).to_equal(0)
        expect(stats.errors).to_equal(0)

    @test
    def test_create_with_values(self):
        """Test creating stats with values."""
        stats = TestStats(
            total=10,
            passed=8,
            failed=1,
            skipped=1,
            errors=0
        )
        expect(stats.total).to_equal(10)
        expect(stats.passed).to_equal(8)
        expect(stats.failed).to_equal(1)
        expect(stats.skipped).to_equal(1)


class TestValidationResult(TestSuite):
    """Test ValidationResult dataclass."""

    @test
    def test_create_validation_result(self):
        """Test creating validation result."""
        result = ValidationResult(
            file_path="test.py",
            pytest_available=True,
            pytest_stats=TestStats(total=5, passed=5),
            cclab_stats=TestStats(total=5, passed=5),
            tests_match=True,
            issues=[]
        )

        expect(result.file_path).to_equal("test.py")
        expect(result.tests_match).to_equal(True)
        expect(len(result.issues)).to_equal(0)

    @test
    def test_validation_with_mismatch(self):
        """Test validation result with mismatch."""
        result = ValidationResult(
            file_path="test.py",
            pytest_available=True,
            pytest_stats=TestStats(total=5, passed=5),
            cclab_stats=TestStats(total=5, passed=4),
            tests_match=False,
            issues=["Passed count mismatch"]
        )

        expect(result.tests_match).to_equal(False)
        expect(len(result.issues)).to_equal(1)


class TestValidationReport(TestSuite):
    """Test ValidationReport class."""

    @test
    def test_empty_report(self):
        """Test empty report."""
        report = ValidationReport()
        expect(report.total_files).to_equal(0)
        expect(report.matching_files).to_equal(0)
        expect(report.mismatched_files).to_equal(0)

    @test
    def test_add_result(self):
        """Test adding results to report."""
        report = ValidationReport()

        result1 = ValidationResult(
            file_path="test1.py",
            pytest_available=True,
            pytest_stats=TestStats(total=5, passed=5),
            cclab_stats=TestStats(total=5, passed=5),
            tests_match=True,
            issues=[]
        )

        result2 = ValidationResult(
            file_path="test2.py",
            pytest_available=True,
            pytest_stats=TestStats(total=3, passed=3),
            cclab_stats=TestStats(total=3, passed=2),
            tests_match=False,
            issues=["Passed count mismatch"]
        )

        report.add_result(result1)
        report.add_result(result2)

        expect(report.total_files).to_equal(2)
        expect(report.matching_files).to_equal(1)
        expect(report.mismatched_files).to_equal(1)
        expect(len(report.all_issues)).to_equal(1)

    @test
    def test_to_dict(self):
        """Test converting report to dictionary."""
        report = ValidationReport()

        result = ValidationResult(
            file_path="test.py",
            pytest_available=True,
            pytest_stats=TestStats(total=5, passed=5),
            cclab_stats=TestStats(total=5, passed=5),
            tests_match=True,
            issues=[]
        )

        report.add_result(result)

        data = report.to_dict()

        expect("summary" in data).to_equal(True)
        expect("results" in data).to_equal(True)
        expect(data["summary"]["total_files"]).to_equal(1)
        expect(data["summary"]["matching_files"]).to_equal(1)

    @test
    def test_save_json(self):
        """Test saving report as JSON."""
        report = ValidationReport()

        result = ValidationResult(
            file_path="test.py",
            pytest_available=True,
            pytest_stats=TestStats(total=5, passed=5),
            cclab_stats=TestStats(total=5, passed=5),
            tests_match=True,
            issues=[]
        )

        report.add_result(result)

        # Save to temporary file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            temp_path = Path(f.name)

        try:
            report.save_json(temp_path)

            # Verify file exists and has content
            expect(temp_path.exists()).to_equal(True)

            import json
            with open(temp_path, 'r') as f:
                data = json.load(f)

            expect("summary" in data).to_equal(True)
            expect(data["summary"]["total_files"]).to_equal(1)

        finally:
            # Clean up
            temp_path.unlink()


# Run tests if executed directly
if __name__ == '__main__':
    from cclab.probe import run_suite
    report1 = run_suite(TestTestStats, verbose=True)
    report2 = run_suite(TestValidationResult, verbose=True)
    report3 = run_suite(TestValidationReport, verbose=True)

    # Exit with error code if tests failed
    import sys
    total_failed = (report1.summary.failed +
                   report2.summary.failed +
                   report3.summary.failed)
    sys.exit(0 if total_failed == 0 else 1)
