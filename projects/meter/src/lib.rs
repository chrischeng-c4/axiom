// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-lib-rs.md#source
// CODEGEN-BEGIN
//! meter: Rust profiling + security issue finder
//!
//! `meter` finds where Rust code is slow and where it is unsafe — it is NOT a test
//! framework (it delegates `cargo test`/`nextest`). It composes two modes into
//! one self-describing report:
//! - **embed** (埋点) — in-process probes: boundary tracing, phase/memory
//!   profiling, benchmarking, baseline regression detection.
//! - **capture** (擷取) — observe a workload from the outside: stack sampling,
//!   `cargo audit` security scanning, fuzzing, and test delegation.
//!
//! The agent-first surface lives in [`report`] (the [`MeterReport`] envelope, the
//! `Finding` schema, and the `IntoFindings` producer trait); the `meter-cli` crate
//! drives it as JSON-on-stdout-by-default verbs.
//!
//! # Example (embed, as a library)
//!
//! ```ignore
//! use meter::performance::BoundaryTracer;
//! let tracer = BoundaryTracer::new();
//! // ... record phases, then fold into a report finding via report::IntoFindings.
//! ```

pub mod agent_eval;
pub mod assertions;
pub mod baseline;
pub mod benchmark;
pub mod discovery;
pub mod fixtures;
pub mod hooks;
pub mod http_server;
pub mod parametrize;
pub mod performance;
pub mod plugin;
pub mod reporter;
pub mod runner;
pub mod rust_runner;
pub mod security;
pub mod ts_runner;

/// Agent-first report layer (always compiled): the `MeterReport` envelope, the
/// `Finding` schema, the `IntoFindings` producer trait, builder, emit, persist,
/// env detection, and offline schema/catalog self-describers.
pub mod report;

/// Capture-mode populators (擷取) — observe workloads from the outside. Gated
/// behind the `capture` feature so the engine rlib stays spawn-free for pure
/// library consumers.
#[cfg(feature = "capture")]
pub mod capture;

// Re-export main types
pub use agent_eval::{
    AgentEvalMetrics, AgentEvalResult, AgentEvaluator, AgentTestCase, CorrectnessResult,
    CostCalculator, CostMetrics, DatasetGitIntegration, DatasetMetadata, DatasetSnapshot,
    ExpectedToolCall, GoldenDataset, LLMJudge, LLMJudgeConfig, LLMJudgeResponse, LatencyMetrics,
    ModelPricing, PricingRegistry, QualityCriterion, QualityScores, ToolAccuracyResult,
};
pub use assertions::{expect, AssertionError, AssertionResult, Expectation};
pub use baseline::{
    BaselineMetadata, BaselineSnapshot, FileBaselineStore, GitMetadata, Improvement,
    PercentileType, Regression, RegressionDetector, RegressionReport, RegressionSeverity,
    RegressionSummary, RegressionThresholds,
};
pub use benchmark::{
    compare_results, print_comparison_table, BenchmarkConfig, BenchmarkEnvironment,
    BenchmarkReport, BenchmarkReportGroup, BenchmarkResult, BenchmarkStats, Benchmarker,
};
pub use discovery::{
    filter_files, walk_files, BenchmarkRegistry, DiscoveryConfig, DiscoveryStats, FileInfo,
    FileType, TestRegistry,
};
pub use fixtures::{FixtureMeta, FixtureRegistry, FixtureScope};
pub use hooks::HookType;
pub use http_server::{RouteConfig, TestServer, TestServerConfig, TestServerHandle};
pub use parametrize::{Parameter, ParameterSet, ParameterValue, ParametrizedTest};
pub use plugin::{
    FilterPlugin, HookError, HookResult, HookSpec, LoggingPlugin, Plugin, PluginConfig,
    PluginManager, TimeoutPlugin,
};

// Re-export performance types (from performance module)
pub use performance::{
    // Profiling
    generate_flamegraph_svg,
    get_rss_bytes,
    BoundaryMetrics,
    BoundaryTiming,
    // Boundary tracing
    BoundaryTracer,
    FlamegraphData,
    GilContentionResult,
    GilTestConfig,
    MemoryProfile,
    MemorySnapshot,
    PhaseBreakdown,
    PhaseTiming,
    ProfileConfig,
    ProfilePhase,
    ProfileResult,
    Profiler,
};

pub use reporter::{
    CoverageInfo, EnvironmentInfo, FileCoverage, ReportFormat, Reporter, TestReport,
};
pub use runner::{Language, TestMeta, TestResult, TestRunner, TestStatus, TestSummary, TestType};
pub use rust_runner::{AuditResult, RustRunner, RustRunnerConfig, RustRunnerResult, Vulnerability};
pub use security::{
    AsyncFuzzConfig, AsyncFuzzer, FuzzConfig, FuzzCrash, FuzzResult, Fuzzer, InjectionResult,
    InjectionTest, MutationStrategy, PayloadCategory, PayloadDatabase, SqlInjectionTester,
};
pub use ts_runner::{NpmAuditResult, TsRunner, TsRunnerConfig, TsRunnerResult, V8Metrics};

// Re-export the agent-first report envelope + finding surface.
pub use report::{
    catalog, json_schema, Completion, EnvBlock, Finding, FindingsSummary, IntoFindings, Invoke,
    Kind, Location, OverallStatus, MeterReport, ReportBuilder, RunnerRecord, Severity, SCHEMA_VERSION,
};
// CODEGEN-END

// HANDWRITE-BEGIN gap="missing-generator:meter-ast-instrumentation" reason="AST-assisted probe-point discovery (feature `ast`) backed by compass (libs/compass), filling the README's 'AST-assisted instrumentation is planned but not implemented' gap. Promote to CODEGEN by adding the instrument module to the lib.rs TD Source snapshot + symbols table."
#[cfg(feature = "ast")]
pub mod instrument;
// HANDWRITE-END
