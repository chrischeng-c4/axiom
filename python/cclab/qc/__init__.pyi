from typing import Any

__all__ = ["FileCoverage", "CoverageInfo", "TestMeta", "TestResult", "TestSummary", "Expectation", "Reporter", "TestReport", "BenchmarkStats", "BenchmarkResult", "BenchmarkConfig", "BenchmarkEnvironment", "BenchmarkReportGroup", "BenchmarkReport", "FileInfo", "DiscoveryConfig", "TestRegistry", "BenchmarkRegistry", "DiscoveryStats", "ParameterValue", "ParameterSet", "Parameter", "ParametrizedTest", "TestServerHandle", "TestServer", "PyTestType", "PyTestStatus", "PyReportFormat", "PyFileType", "PyProfilePhase", "PyFixtureScope", "PyHookType", "HookRegistry", "TestRunner", "FixtureRegistry", "FixtureMeta", "PhaseTiming", "PhaseBreakdown", "GilTestConfig", "GilContentionResult", "MemorySnapshot", "MemoryProfile", "FlamegraphData", "ProfileResult", "ProfileConfig", "expect", "compare_benchmarks", "print_comparison_table", "discover_files", "filter_files_by_pattern", "generate_flamegraph"]

class FileCoverage:
    """Python FileCoverage class - coverage info for a single file"""
    def __init__(self, path: str, statements: int, covered: int, missing_lines: list[int]) -> None:
        ...
    @property
    def path(self) -> str:
        ...
    @property
    def statements(self) -> int:
        ...
    @property
    def covered(self) -> int:
        ...
    @property
    def missing_lines(self) -> list[int]:
        ...
    @property
    def coverage_percent(self) -> float:
        ...
    def __repr__(self) -> str:
        ...

class CoverageInfo:
    """Python CoverageInfo class - overall coverage summary"""
    def __init__(self, total_statements: int = 0, covered_statements: int = 0, files: list[Any] | None = None, uncovered_files: list[str] | None = None) -> None:
        ...
    @property
    def total_statements(self) -> int:
        ...
    @property
    def covered_statements(self) -> int:
        ...
    @property
    def coverage_percent(self) -> float:
        ...
    @property
    def files(self) -> list[Any]:
        ...
    @property
    def uncovered_files(self) -> list[str]:
        ...
    def add_file(self, file: Any) -> None:
        """Add file coverage"""
        ...
    def add_uncovered_file(self, path: str) -> None:
        """Add uncovered file"""
        ...
    def __repr__(self) -> str:
        ...

class TestMeta:
    """Python TestMeta class - metadata for a test"""
    def __init__(self, name: str, test_type: Any | None = None, timeout: float | None = None, tags: list[str] | None = None) -> None:
        """Create new test metadata"""
        ...
    @property
    def name(self) -> str:
        """Test name"""
        ...
    @property
    def full_name(self) -> str:
        """Full qualified name"""
        ...
    @full_name.setter
    def full_name(self, full_name: str) -> None:
        """Set full name"""
        ...
    @property
    def test_type(self) -> Any:
        """Test type"""
        ...
    @property
    def timeout(self) -> float | None:
        """Timeout in seconds"""
        ...
    @property
    def tags(self) -> list[str]:
        """Tags"""
        ...
    @property
    def skip_reason(self) -> str | None:
        """Skip reason"""
        ...
    def is_skipped(self) -> bool:
        """Check if skipped"""
        ...
    def has_tag(self, tag: str) -> bool:
        """Check if has tag"""
        ...
    def skip(self, reason: str) -> None:
        """Skip this test"""
        ...
    def set_file_path(self, path: str) -> None:
        """Set source file path"""
        ...
    def set_line_number(self, line: int) -> None:
        """Set line number"""
        ...
    def __repr__(self) -> str:
        ...

class TestResult:
    """Python TestResult class"""
    @staticmethod
    def passed(meta: Any, duration_ms: int) -> Any:
        """Create a passed result"""
        ...
    @staticmethod
    def failed(meta: Any, duration_ms: int, error: str) -> Any:
        """Create a failed result"""
        ...
    @staticmethod
    def skipped(meta: Any, reason: str) -> Any:
        """Create a skipped result"""
        ...
    @staticmethod
    def error(meta: Any, duration_ms: int, error: str) -> Any:
        """Create an error result"""
        ...
    @property
    def meta(self) -> Any:
        """Test metadata"""
        ...
    @property
    def status(self) -> Any:
        """Test status"""
        ...
    @property
    def duration_ms(self) -> int:
        """Duration in milliseconds"""
        ...
    @property
    def error_message(self) -> str | None:
        """Error message"""
        ...
    @property
    def stack_trace(self) -> str | None:
        """Stack trace"""
        ...
    def set_stack_trace(self, trace: str) -> None:
        """Set stack trace"""
        ...
    def is_passed(self) -> bool:
        """Check if passed"""
        ...
    def is_failed(self) -> bool:
        """Check if failed"""
        ...
    def __repr__(self) -> str:
        ...

class TestSummary:
    """Python TestSummary class"""
    @property
    def total(self) -> int:
        """Total tests"""
        ...
    @property
    def passed(self) -> int:
        """Passed tests"""
        ...
    @property
    def failed(self) -> int:
        """Failed tests"""
        ...
    @property
    def skipped(self) -> int:
        """Skipped tests"""
        ...
    @property
    def errors(self) -> int:
        """Error tests"""
        ...
    @property
    def total_duration_ms(self) -> int:
        """Total duration in ms"""
        ...
    def all_passed(self) -> bool:
        """Check if all passed"""
        ...
    def pass_rate(self) -> float:
        """Get pass rate"""
        ...
    def __repr__(self) -> str:
        ...

class Expectation:
    """Python Expectation class for fluent assertions"""
    def to_equal(self, expected: Any) -> None:
        """Assert equality"""
        ...
    def to_not_equal(self, expected: Any) -> None:
        """Assert not equal"""
        ...
    def to_be_truthy(self) -> None:
        """Assert truthy (non-null, non-empty, non-false)"""
        ...
    def to_be_falsy(self) -> None:
        """Assert falsy (null, empty, false, zero)"""
        ...
    def to_be_none(self) -> None:
        """Assert None/null"""
        ...
    def to_not_be_none(self) -> None:
        """Assert not None/null"""
        ...
    def to_be_greater_than(self, expected: Any) -> None:
        """Assert greater than"""
        ...
    def to_be_less_than(self, expected: Any) -> None:
        """Assert less than"""
        ...
    def to_contain(self, expected: Any) -> None:
        """Assert contains"""
        ...
    def to_have_length(self, expected: int) -> None:
        """Assert has length"""
        ...
    def __repr__(self) -> str:
        ...

class Reporter:
    """Python Reporter class"""
    def __init__(self, format: Any = PyReportFormat::Markdown) -> None:
        """Create a new reporter"""
        ...
    @staticmethod
    def markdown() -> Any:
        """Create markdown reporter"""
        ...
    @staticmethod
    def html() -> Any:
        """Create HTML reporter"""
        ...
    @staticmethod
    def json() -> Any:
        """Create JSON reporter"""
        ...
    @staticmethod
    def junit() -> Any:
        """Create JUnit reporter"""
        ...
    def generate(self, report: Any) -> str:
        """Generate report string"""
        ...

class TestReport:
    """Python TestReport class"""
    def __init__(self, suite_name: str, results: list[Any]) -> None:
        """Create a new test report"""
        ...
    @property
    def suite_name(self) -> str:
        """Suite name"""
        ...
    @property
    def generated_at(self) -> str:
        """Generated timestamp"""
        ...
    @property
    def duration_ms(self) -> int:
        """Duration in milliseconds"""
        ...
    @property
    def summary(self) -> Any:
        """Summary"""
        ...
    @property
    def results(self) -> list[Any]:
        """All results"""
        ...
    def results_by_type(self, test_type: Any) -> list[Any]:
        """Get results by test type"""
        ...
    def failed_results(self) -> list[Any]:
        """Get failed results"""
        ...
    def set_environment(self, python_version: str | None, rust_version: str | None, platform: str | None, hostname: str | None) -> None:
        """Set environment info"""
        ...
    def set_coverage(self, coverage: Any) -> None:
        """Set coverage info"""
        ...
    @property
    def coverage(self) -> Any | None:
        """Get coverage info"""
        ...

class BenchmarkStats:
    """Python BenchmarkStats class"""
    @property
    def iterations(self) -> int:
        """Number of iterations per round"""
        ...
    @property
    def rounds(self) -> int:
        """Number of rounds"""
        ...
    @property
    def warmup(self) -> int:
        """Number of warmup iterations"""
        ...
    @property
    def total_runs(self) -> int:
        """Total number of timed runs"""
        ...
    @property
    def mean_ms(self) -> float:
        """Mean time per operation (ms)"""
        ...
    @property
    def min_ms(self) -> float:
        """Minimum time observed (ms)"""
        ...
    @property
    def max_ms(self) -> float:
        """Maximum time observed (ms)"""
        ...
    @property
    def stddev_ms(self) -> float:
        """Standard deviation (ms)"""
        ...
    @property
    def median_ms(self) -> float:
        """Median time (ms)"""
        ...
    @property
    def total_ms(self) -> float:
        """Total time for all runs (ms)"""
        ...
    @property
    def all_times_ms(self) -> list[float]:
        """All individual timings (ms)"""
        ...
    @property
    def p25_ms(self) -> float:
        """25th percentile (Q1)"""
        ...
    @property
    def p75_ms(self) -> float:
        """75th percentile (Q3)"""
        ...
    @property
    def p95_ms(self) -> float:
        """95th percentile"""
        ...
    @property
    def p99_ms(self) -> float:
        """99th percentile"""
        ...
    @property
    def iqr_ms(self) -> float:
        """Interquartile range (Q3 - Q1)"""
        ...
    @property
    def outliers(self) -> int:
        """Total number of outliers"""
        ...
    @property
    def outliers_low(self) -> int:
        """Outliers below Q1 - 1.5*IQR"""
        ...
    @property
    def outliers_high(self) -> int:
        """Outliers above Q3 + 1.5*IQR"""
        ...
    @property
    def std_error_ms(self) -> float:
        """Standard error (stddev / sqrt(n))"""
        ...
    @property
    def ci_lower_ms(self) -> float:
        """95% CI lower bound"""
        ...
    @property
    def ci_upper_ms(self) -> float:
        """95% CI upper bound"""
        ...
    def ops_per_second(self) -> float:
        """Calculate operations per second"""
        ...
    def format(self) -> str:
        """Format stats as human-readable string"""
        ...
    def format_short(self) -> str:
        """Format stats as short single-line summary"""
        ...
    def __repr__(self) -> str:
        ...

class BenchmarkResult:
    """Python BenchmarkResult class"""
    @staticmethod
    def from_times(name: str, times_ms: list[float], iterations: int = 20, rounds: int = 3, warmup: int = 3) -> Any:
        """Create a new benchmark result from collected times"""
        ...
    @staticmethod
    def failure(name: str, error: str) -> Any:
        """Create a failed benchmark result"""
        ...
    @property
    def name(self) -> str:
        """Name of this benchmark"""
        ...
    @property
    def stats(self) -> Any:
        """Timing statistics"""
        ...
    @property
    def success(self) -> bool:
        """Whether benchmark completed successfully"""
        ...
    @property
    def error(self) -> str | None:
        """Error message if failed"""
        ...
    def format(self) -> str:
        """Format result as human-readable string"""
        ...
    def print_detailed(self) -> None:
        """Print detailed statistics to stdout"""
        ...
    def __repr__(self) -> str:
        ...

class BenchmarkConfig:
    """Python BenchmarkConfig class"""
    def __init__(self, iterations: int = 20, rounds: int = 3, warmup: int = 3) -> None:
        """Create a new benchmark configuration"""
        ...
    @staticmethod
    def quick() -> Any:
        """Create a quick benchmark configuration"""
        ...
    @staticmethod
    def thorough() -> Any:
        """Create a thorough benchmark configuration"""
        ...
    @staticmethod
    def calibrated(sample_time_ms: float, target_time_ms: float = 100.0) -> Any:
        """Create a calibrated benchmark configuration"""
        ...
    @property
    def iterations(self) -> int:
        """Number of iterations per round"""
        ...
    @property
    def rounds(self) -> int:
        """Number of rounds"""
        ...
    @property
    def warmup(self) -> int:
        """Number of warmup iterations"""
        ...
    def __repr__(self) -> str:
        ...

class BenchmarkEnvironment:
    """Python BenchmarkEnvironment class"""
    def __init__(self, python_version: str | None = None, rust_version: str | None = None, platform: str | None = None, cpu: str | None = None, hostname: str | None = None) -> None:
        ...
    @property
    def python_version(self) -> str | None:
        ...
    @property
    def rust_version(self) -> str | None:
        ...
    @property
    def platform(self) -> str | None:
        ...
    @property
    def cpu(self) -> str | None:
        ...
    @property
    def hostname(self) -> str | None:
        ...

class BenchmarkReportGroup:
    """Python BenchmarkReportGroup class"""
    def __init__(self, name: str, baseline: str | None = None) -> None:
        ...
    @property
    def name(self) -> str:
        ...
    @property
    def baseline(self) -> str | None:
        ...
    @property
    def results(self) -> list[Any]:
        ...
    def add_result(self, result: Any) -> None:
        """Add a result to this group"""
        ...

class BenchmarkReport:
    """Python BenchmarkReport class"""
    def __init__(self, title: str, description: str | None = None) -> None:
        ...
    @property
    def title(self) -> str:
        ...
    @property
    def description(self) -> str | None:
        ...
    @property
    def generated_at(self) -> str:
        ...
    @property
    def total_duration_ms(self) -> float:
        ...
    @property
    def groups(self) -> list[Any]:
        ...
    def add_group(self, group: Any) -> None:
        """Add a benchmark group"""
        ...
    def set_environment(self, env: Any) -> None:
        """Set environment info"""
        ...
    def to_json(self) -> str:
        """Generate JSON report"""
        ...
    def to_html(self) -> str:
        """Generate HTML report with charts"""
        ...
    def to_markdown(self) -> str:
        """Generate Markdown report"""
        ...
    def to_yaml(self) -> str:
        """Generate YAML report"""
        ...
    def to_console(self) -> str:
        """Generate console output with ANSI colors"""
        ...
    def save(self, path: str, format: str) -> None:
        """Save report to file"""
        ...

class FileInfo:
    """Python FileInfo wrapper"""
    @property
    def path(self) -> str:
        ...
    @property
    def module_name(self) -> str:
        ...
    @property
    def file_type(self) -> Any:
        ...
    def __repr__(self) -> str:
        ...

class DiscoveryConfig:
    """Python DiscoveryConfig wrapper"""
    def __init__(self, root_path: str = "tests/", patterns: list[str] | None = None, exclusions: list[str] | None = None, max_depth: int = 10) -> None:
        ...
    @property
    def root_path(self) -> str:
        ...
    @property
    def patterns(self) -> list[str]:
        ...
    @property
    def exclusions(self) -> list[str]:
        ...
    @property
    def max_depth(self) -> int:
        ...
    def __repr__(self) -> str:
        ...

class TestRegistry:
    """Python TestRegistry wrapper"""
    def __init__(self) -> None:
        ...
    def register(self, file: Any) -> None:
        ...
    def get_all(self) -> list[Any]:
        ...
    def filter_by_pattern(self, pattern: str) -> None:
        ...
    def count(self) -> int:
        ...
    def clear(self) -> None:
        ...
    def __repr__(self) -> str:
        ...

class BenchmarkRegistry:
    """Python BenchmarkRegistry wrapper"""
    def __init__(self) -> None:
        ...
    def register(self, file: Any) -> None:
        ...
    def get_all(self) -> list[Any]:
        ...
    def filter_by_pattern(self, pattern: str) -> None:
        ...
    def count(self) -> int:
        ...
    def clear(self) -> None:
        ...
    def __repr__(self) -> str:
        ...

class DiscoveryStats:
    """Python DiscoveryStats wrapper"""
    @property
    def files_found(self) -> int:
        ...
    @property
    def filtered_count(self) -> int:
        ...
    @property
    def discovery_time_ms(self) -> int:
        ...
    def __repr__(self) -> str:
        ...

class ParameterValue:
    """Python ParameterValue class"""
    @staticmethod
    def int(value: int) -> Any:
        """Create an integer parameter value"""
        ...
    @staticmethod
    def float(value: float) -> Any:
        """Create a float parameter value"""
        ...
    @staticmethod
    def string(value: str) -> Any:
        """Create a string parameter value"""
        ...
    @staticmethod
    def bool(value: bool) -> Any:
        """Create a boolean parameter value"""
        ...
    @staticmethod
    def none() -> Any:
        """Create a None parameter value"""
        ...
    @staticmethod
    def from_py(obj: Any) -> Any:
        """Create from Python object (auto-conversion)"""
        ...
    def format_for_name(self) -> str:
        """Format for test name"""
        ...
    def to_py(self) -> Any:
        """Convert to Python object"""
        ...
    def __str__(self) -> str:
        ...
    def __repr__(self) -> str:
        ...

class ParameterSet:
    """Python ParameterSet class"""
    def __init__(self) -> None:
        ...
    def add(self, name: str, value: Any) -> None:
        """Add a parameter"""
        ...
    def get(self, name: str) -> Any | None:
        """Get a parameter value"""
        ...
    def format_for_name(self) -> str:
        """Format for test name"""
        ...
    def to_dict(self) -> Any:
        """Convert to Python dict"""
        ...
    def __len__(self) -> int:
        ...
    def __repr__(self) -> str:
        ...

class Parameter:
    """Python Parameter class"""
    def __init__(self, name: str, values: list[Any]) -> None:
        ...
    @property
    def name(self) -> str:
        ...
    @property
    def values(self) -> list[Any]:
        ...
    def validate(self) -> None:
        """Validate the parameter"""
        ...
    def __repr__(self) -> str:
        ...

class ParametrizedTest:
    """Python ParametrizedTest class"""
    def __init__(self, base_name: str) -> None:
        ...
    def add_parameter(self, param: Any) -> None:
        """Add a parameter"""
        ...
    def expand(self) -> list[tuple[str, Any]]:
        """Expand into test instances"""
        ...
    def count_instances(self) -> int:
        """Count total instances"""
        ...
    @property
    def base_name(self) -> str:
        ...
    def __repr__(self) -> str:
        ...

class TestServerHandle:
    """Python wrapper for TestServerHandle"""
    @property
    def url(self) -> str:
        """Get the base URL for this server"""
        ...
    @property
    def port(self) -> int:
        """Get the port number"""
        ...
    @property
    def client(self) -> str:
        """Get an HTTP client for making requests (returns URL for now)"""
        ...
    def stop(self) -> None:
        """Stop the server"""
        ...
    def __repr__(self) -> str:
        ...

class TestServer:
    """Python TestServer class for creating test HTTP servers"""
    def __init__(self) -> None:
        """Create a new test server"""
        ...
    @staticmethod
    def from_app(app_module: str, app_callable: str = "app", port: int = 18765, startup_timeout: float = 10.0, health_endpoint: str | None = None) -> Any:
        ...
    def port(self, port: int) -> None:
        """Set the port to listen on"""
        ...
    def get(self, path: str, response: Any) -> None:
        """Add a GET route with JSON response"""
        ...
    def routes(self, routes: dict[Any, Any]) -> None:
        """Add multiple routes from a dict"""
        ...
    def start(self) -> Any:
        """Start the server (async)"""
        ...
    def __repr__(self) -> str:
        ...

class PyTestType:
    def __str__(self) -> str:
        ...
    def __repr__(self) -> str:
        ...

class PyTestStatus:
    def __str__(self) -> str:
        ...
    def __repr__(self) -> str:
        ...

class PyReportFormat:
    def __str__(self) -> str:
        ...
    def __repr__(self) -> str:
        ...

class PyFileType:
    def __str__(self) -> str:
        ...
    def __repr__(self) -> str:
        ...

class PyProfilePhase:
    def __str__(self) -> str:
        ...
    def __repr__(self) -> str:
        ...

class PyFixtureScope:
    def __str__(self) -> str:
        ...
    def __repr__(self) -> str:
        ...
    @staticmethod
    def from_string(s: str) -> Any:
        ...

class PyHookType:
    def __str__(self) -> str:
        ...
    def __repr__(self) -> str:
        ...

class HookRegistry:
    """Python HookRegistry class"""
    def __init__(self) -> None:
        """Create a new hook registry"""
        ...
    def register_hook(self, hook_type: Any, hook_fn: Any) -> None:
        """Register a hook function"""
        ...
    def clear_hooks(self, hook_type: Any) -> None:
        """Clear all hooks of a specific type"""
        ...
    def clear_all(self) -> None:
        """Clear all hooks"""
        ...
    def hook_count(self, hook_type: Any) -> int:
        """Get the number of registered hooks for a specific type"""
        ...
    def run_hooks(self, hook_type: Any, suite_instance: Any | None) -> Any:
        """Run hooks of a specific type (async method)"""
        ...
    def __repr__(self) -> str:
        ...

class TestRunner:
    """Python TestRunner class"""
    def __init__(self, test_type: Any | None = None, tags: list[str] | None = None, name_pattern: str | None = None, fail_fast: bool = False, verbose: bool = False, parallel: bool = False, max_workers: int = 4) -> None:
        ...
    def start(self) -> None:
        """Start the test run"""
        ...
    def record(self, result: Any) -> None:
        """Record a test result"""
        ...
    def results(self) -> list[Any]:
        """Get all results"""
        ...
    def summary(self) -> Any:
        """Get summary"""
        ...
    def total_duration_secs(self) -> float:
        """Get total duration in seconds"""
        ...
    def should_run(self, meta: Any) -> bool:
        """Check if test should run based on filters"""
        ...
    def run_parallel_async(self, suite_instance: Any, test_descriptors: list[Any]) -> Any:
        """Run tests in parallel using Tokio (returns Python awaitable)"""
        ...

class FixtureRegistry:
    """Python FixtureRegistry wrapper"""
    def __init__(self) -> None:
        ...
    def register(self, name: str, scope: Any, autouse: bool, dependencies: list[str], has_teardown: bool) -> None:
        """Register a fixture from Python"""
        ...
    def get_meta(self, name: str) -> Any | None:
        """Get fixture metadata by name"""
        ...
    def get_all_names(self) -> list[str]:
        """Get all fixture names"""
        ...
    def get_autouse_fixtures(self, scope: Any) -> list[str]:
        """Get autouse fixtures for a scope"""
        ...
    def resolve_order(self, fixture_names: list[str]) -> list[str]:
        """Resolve fixture dependency order"""
        ...
    def detect_circular_deps(self) -> None:
        """Detect circular dependencies"""
        ...
    def has_fixture(self, name: str) -> bool:
        """Check if fixture exists"""
        ...
    def __len__(self) -> int:
        """Get number of registered fixtures"""
        ...

class FixtureMeta:
    """Python wrapper for FixtureMeta"""
    name: str
    scope: Any
    autouse: bool
    dependencies: list[str]
    has_teardown: bool
    def __repr__(self) -> str:
        ...

class PhaseTiming:
    """Python PhaseTiming class"""
    @property
    def total_ns(self) -> int:
        """Total time in nanoseconds"""
        ...
    @property
    def count(self) -> int:
        """Number of samples"""
        ...
    @property
    def min_ns(self) -> int:
        """Minimum time in nanoseconds"""
        ...
    @property
    def max_ns(self) -> int:
        """Maximum time in nanoseconds"""
        ...
    @property
    def avg_ns(self) -> float:
        """Average time in nanoseconds"""
        ...
    @property
    def total_ms(self) -> float:
        """Total time in milliseconds"""
        ...
    @property
    def avg_ms(self) -> float:
        """Average time in milliseconds"""
        ...
    def __repr__(self) -> str:
        ...

class PhaseBreakdown:
    """Python PhaseBreakdown class"""
    def get_phase(self, phase_name: str) -> Any | None:
        """Get timing for a specific phase"""
        ...
    def phase_names(self) -> list[str]:
        """Get all phase names"""
        ...
    @property
    def operation_count(self) -> int:
        """Get operation count"""
        ...
    @property
    def total_duration_ms(self) -> float:
        """Get total duration in milliseconds"""
        ...
    def percentage_breakdown(self) -> Any:
        """Get percentage breakdown"""
        ...
    def format(self) -> str:
        """Format as human-readable string"""
        ...
    def __repr__(self) -> str:
        ...

class GilTestConfig:
    """Python GilTestConfig class"""
    def __init__(self, concurrent_workers: int = 4, duration_secs: float = 10.0, operations_per_worker: int = 100, warmup_iterations: int = 3) -> None:
        ...
    @property
    def concurrent_workers(self) -> int:
        ...
    @property
    def duration_secs(self) -> float:
        ...
    @property
    def operations_per_worker(self) -> int:
        ...
    @property
    def warmup_iterations(self) -> int:
        ...
    def __repr__(self) -> str:
        ...

class GilContentionResult:
    """Python GilContentionResult class"""
    @property
    def sequential_baseline_ms(self) -> float:
        ...
    @property
    def concurrent_total_ms(self) -> float:
        ...
    @property
    def worker_times_ms(self) -> list[float]:
        ...
    @property
    def overhead_percent(self) -> float:
        ...
    @property
    def gil_release_effective(self) -> bool:
        ...
    @property
    def theoretical_speedup(self) -> float:
        ...
    @property
    def actual_speedup(self) -> float:
        ...
    @property
    def efficiency_percent(self) -> float:
        ...
    def format(self) -> str:
        """Format as human-readable string"""
        ...
    def __repr__(self) -> str:
        ...

class MemorySnapshot:
    """Python MemorySnapshot class"""
    @property
    def rss_bytes(self) -> int:
        ...
    @property
    def peak_rss_bytes(self) -> int:
        ...
    @property
    def rss_mb(self) -> float:
        ...
    @property
    def peak_rss_mb(self) -> float:
        ...
    def __repr__(self) -> str:
        ...

class MemoryProfile:
    """Python MemoryProfile class"""
    @property
    def before(self) -> Any:
        ...
    @property
    def after(self) -> Any:
        ...
    @property
    def peak(self) -> Any:
        ...
    @property
    def delta_bytes(self) -> int:
        ...
    @property
    def delta_mb(self) -> float:
        ...
    @property
    def peak_rss_mb(self) -> float:
        ...
    @property
    def iterations(self) -> int:
        ...
    def format(self) -> str:
        """Format as human-readable string"""
        ...
    def __repr__(self) -> str:
        ...

class FlamegraphData:
    """Python FlamegraphData class"""
    def __init__(self) -> None:
        ...
    def add_stack(self, stack: str) -> None:
        """Add a folded stack sample"""
        ...
    @property
    def folded_stacks(self) -> list[str]:
        ...
    @property
    def sample_count(self) -> int:
        ...
    def has_data(self) -> bool:
        """Check if there's data"""
        ...
    def __repr__(self) -> str:
        ...

class ProfileResult:
    """Python ProfileResult class"""
    @property
    def name(self) -> str:
        ...
    @property
    def started_at(self) -> str:
        ...
    @property
    def ended_at(self) -> str:
        ...
    @property
    def duration_ms(self) -> float:
        ...
    @property
    def success(self) -> bool:
        ...
    @property
    def error(self) -> str | None:
        ...
    @property
    def phase_breakdown(self) -> Any | None:
        ...
    @property
    def gil_analysis(self) -> Any | None:
        ...
    @property
    def memory_profile(self) -> Any | None:
        ...
    @property
    def flamegraph(self) -> Any | None:
        ...
    def format(self) -> str:
        """Format as human-readable string"""
        ...
    def to_json(self) -> str:
        """Export to JSON"""
        ...
    def __repr__(self) -> str:
        ...

class ProfileConfig:
    """Python ProfileConfig class"""
    def __init__(self, enable_phase_breakdown: bool = True, enable_gil_analysis: bool = False, enable_memory_profile: bool = False, enable_flamegraph: bool = False, iterations: int = 100, warmup: int = 10, output_dir: str | None = None) -> None:
        ...
    @staticmethod
    def full() -> Any:
        """Create full profiling config"""
        ...
    @staticmethod
    def quick() -> Any:
        """Create quick profiling config"""
        ...
    @property
    def enable_phase_breakdown(self) -> bool:
        ...
    @property
    def enable_gil_analysis(self) -> bool:
        ...
    @property
    def enable_memory_profile(self) -> bool:
        ...
    @property
    def enable_flamegraph(self) -> bool:
        ...
    @property
    def iterations(self) -> int:
        ...
    @property
    def warmup(self) -> int:
        ...
    @property
    def output_dir(self) -> str | None:
        ...
    @property
    def gil_config(self) -> Any:
        ...
    def with_gil_config(self, config: Any) -> Any:
        """Set GIL test configuration"""
        ...
    def with_output_dir(self, dir: str) -> Any:
        """Set output directory"""
        ...
    def __repr__(self) -> str:
        ...

def expect(actual: Any) -> Any:
    """Create an expectation from a Python value"""
    ...

def compare_benchmarks(results: list[Any], baseline_name: str | None = None) -> str:
    """Compare multiple benchmark results and return formatted comparison"""
    ...

def print_comparison_table(results: list[Any], baseline_name: str | None = None) -> None:
    """Print a comparison table to stdout with enhanced statistics"""
    ...

def discover_files(config: Any) -> list[Any]:
    """Walk files and discover test/benchmark files"""
    ...

def filter_files_by_pattern(files: list[Any], pattern: str) -> list[Any]:
    """Filter files by pattern"""
    ...

def generate_flamegraph(folded_stacks: list[str], title: str, output_path: str) -> None:
    """Generate flamegraph SVG from folded stacks"""
    ...

