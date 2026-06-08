// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
// CODEGEN-BEGIN
//! Profiling infrastructure for cclab operations
//!
//! Provides comprehensive profiling capabilities:
//! - Phase breakdown timing (Python extract, Rust convert, Network I/O)
//! - GIL contention analysis under concurrent load
//! - Memory profiling (peak memory, allocation tracking)
//! - Flamegraph generation using inferno crate

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};

// ============================================================================
// Phase Breakdown Types
// ============================================================================

/// Timing phases for a cclab operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
pub enum ProfilePhase {
    /// Python object extraction (GIL held)
    PythonExtract,
    /// Rust BSON conversion (GIL released)
    RustConvert,
    /// Network I/O to MongoDB (GIL released)
    NetworkIO,
    /// binding boundary overhead
    BindingBoundary,
    /// Total operation time
    Total,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
impl std::fmt::Display for ProfilePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfilePhase::PythonExtract => write!(f, "PythonExtract"),
            ProfilePhase::RustConvert => write!(f, "RustConvert"),
            ProfilePhase::NetworkIO => write!(f, "NetworkIO"),
            ProfilePhase::BindingBoundary => write!(f, "BindingBoundary"),
            ProfilePhase::Total => write!(f, "Total"),
        }
    }
}

/// Timing data for a single phase
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
pub struct PhaseTiming {
    /// Total time spent in this phase (nanoseconds)
    pub total_ns: u64,
    /// Number of times this phase was entered
    pub count: u64,
    /// Minimum time for single entry
    pub min_ns: u64,
    /// Maximum time for single entry
    pub max_ns: u64,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
impl PhaseTiming {
    /// Create a new empty phase timing
    pub fn new() -> Self {
        Self {
            total_ns: 0,
            count: 0,
            min_ns: u64::MAX,
            max_ns: 0,
        }
    }

    /// Record a timing sample
    pub fn record(&mut self, duration_ns: u64) {
        self.total_ns += duration_ns;
        self.count += 1;
        self.min_ns = self.min_ns.min(duration_ns);
        self.max_ns = self.max_ns.max(duration_ns);
    }

    /// Get average time in nanoseconds
    pub fn avg_ns(&self) -> f64 {
        if self.count > 0 {
            self.total_ns as f64 / self.count as f64
        } else {
            0.0
        }
    }

    /// Get total time in milliseconds
    pub fn total_ms(&self) -> f64 {
        self.total_ns as f64 / 1_000_000.0
    }

    /// Get average time in milliseconds
    pub fn avg_ms(&self) -> f64 {
        self.avg_ns() / 1_000_000.0
    }
}

/// Complete phase breakdown for a profiling session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
pub struct PhaseBreakdown {
    /// Timing for each phase
    pub phases: HashMap<String, PhaseTiming>,
    /// Total operation count
    pub operation_count: u64,
    /// Total profiling duration in nanoseconds
    pub total_duration_ns: u64,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
impl PhaseBreakdown {
    /// Create a new phase breakdown
    pub fn new() -> Self {
        Self {
            phases: HashMap::new(),
            operation_count: 0,
            total_duration_ns: 0,
        }
    }

    /// Create from collected timing data
    pub fn from_times(
        phase_times: HashMap<String, Vec<u64>>,
        operation_count: u64,
        total_duration_ns: u64,
    ) -> Self {
        let mut phases = HashMap::new();

        for (name, times) in phase_times {
            let mut timing = PhaseTiming::new();
            for t in times {
                timing.record(t);
            }
            phases.insert(name, timing);
        }

        Self {
            phases,
            operation_count,
            total_duration_ns,
        }
    }

    /// Get timing for a specific phase
    pub fn get_phase(&self, phase_name: &str) -> Option<&PhaseTiming> {
        self.phases.get(phase_name)
    }

    /// Get all phase names
    pub fn phase_names(&self) -> Vec<String> {
        self.phases.keys().cloned().collect()
    }

    /// Get percentage breakdown
    pub fn percentage_breakdown(&self) -> HashMap<String, f64> {
        let total = self.total_duration_ns as f64;
        if total == 0.0 {
            return HashMap::new();
        }

        self.phases
            .iter()
            .map(|(name, timing)| (name.clone(), timing.total_ns as f64 / total * 100.0))
            .collect()
    }

    /// Get total duration in milliseconds
    pub fn total_duration_ms(&self) -> f64 {
        self.total_duration_ns as f64 / 1_000_000.0
    }

    /// Format as human-readable string
    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!(
            "Phase Breakdown ({} operations, {:.2}ms total)\n",
            self.operation_count,
            self.total_duration_ms()
        ));
        output.push_str(&"-".repeat(60));
        output.push('\n');

        let total = self.total_duration_ns as f64;

        // Sort phases by total time descending
        let mut phase_vec: Vec<_> = self.phases.iter().collect();
        phase_vec.sort_by(|a, b| b.1.total_ns.cmp(&a.1.total_ns));

        for (name, timing) in phase_vec {
            let pct = if total > 0.0 {
                timing.total_ns as f64 / total * 100.0
            } else {
                0.0
            };
            output.push_str(&format!(
                "{:<20} {:>10.3}ms ({:>5.1}%) [{}x, avg={:.3}ms]\n",
                name,
                timing.total_ms(),
                pct,
                timing.count,
                timing.avg_ms()
            ));
        }
        output
    }
}

// ============================================================================
// GIL Contention Analysis Types
// ============================================================================

/// Configuration for GIL contention testing
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
pub struct GilTestConfig {
    /// Number of concurrent threads/tasks
    pub concurrent_workers: usize,
    /// Duration of the test in seconds
    pub duration_secs: f64,
    /// Operations per worker
    pub operations_per_worker: u64,
    /// Warmup iterations
    pub warmup_iterations: u32,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
impl Default for GilTestConfig {
    fn default() -> Self {
        Self {
            concurrent_workers: 4,
            duration_secs: 10.0,
            operations_per_worker: 100,
            warmup_iterations: 3,
        }
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
impl GilTestConfig {
    /// Create a new config with specified workers
    pub fn with_workers(workers: usize) -> Self {
        Self {
            concurrent_workers: workers,
            ..Default::default()
        }
    }
}

/// Results from GIL contention analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
pub struct GilContentionResult {
    /// Config used for the test
    pub config: GilTestConfig,
    /// Sequential baseline (single thread) in milliseconds
    pub sequential_baseline_ms: f64,
    /// Concurrent total time in milliseconds
    pub concurrent_total_ms: f64,
    /// Per-worker times in milliseconds
    pub worker_times_ms: Vec<f64>,
    /// Calculated overhead percentage
    pub overhead_percent: f64,
    /// Whether GIL release was effective (overhead < 10%)
    pub gil_release_effective: bool,
    /// Theoretical speedup if perfectly parallel
    pub theoretical_speedup: f64,
    /// Actual speedup achieved
    pub actual_speedup: f64,
    /// Efficiency percentage (actual/theoretical)
    pub efficiency_percent: f64,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
impl GilContentionResult {
    /// Create from measurement data
    pub fn from_measurements(
        config: GilTestConfig,
        sequential_ms: f64,
        concurrent_ms: f64,
        worker_times: Vec<f64>,
    ) -> Self {
        let n_workers = config.concurrent_workers as f64;
        let theoretical_speedup = n_workers;
        let actual_speedup = if concurrent_ms > 0.0 {
            sequential_ms / concurrent_ms
        } else {
            0.0
        };
        let efficiency = if theoretical_speedup > 0.0 {
            (actual_speedup / theoretical_speedup) * 100.0
        } else {
            0.0
        };
        let overhead = if sequential_ms > 0.0 {
            ((concurrent_ms / sequential_ms) - 1.0) * 100.0
        } else {
            0.0
        };

        Self {
            config,
            sequential_baseline_ms: sequential_ms,
            concurrent_total_ms: concurrent_ms,
            worker_times_ms: worker_times,
            overhead_percent: overhead,
            gil_release_effective: overhead < 10.0,
            theoretical_speedup,
            actual_speedup,
            efficiency_percent: efficiency,
        }
    }

    /// Format as human-readable string
    pub fn format(&self) -> String {
        format!(
            "GIL Contention Analysis\n\
             {}\n\
             Workers:             {}\n\
             Sequential baseline: {:.3}ms\n\
             Concurrent total:    {:.3}ms\n\
             Overhead:            {:+.1}%\n\
             GIL release:         {}\n\
             Theoretical speedup: {:.2}x\n\
             Actual speedup:      {:.2}x\n\
             Efficiency:          {:.1}%\n",
            "-".repeat(40),
            self.config.concurrent_workers,
            self.sequential_baseline_ms,
            self.concurrent_total_ms,
            self.overhead_percent,
            if self.gil_release_effective {
                "EFFECTIVE"
            } else {
                "BLOCKED"
            },
            self.theoretical_speedup,
            self.actual_speedup,
            self.efficiency_percent
        )
    }
}

// ============================================================================
// Memory Profiling Types
// ============================================================================

/// Memory snapshot at a point in time
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
pub struct MemorySnapshot {
    /// Resident set size (RSS) in bytes
    pub rss_bytes: u64,
    /// Peak RSS in bytes
    pub peak_rss_bytes: u64,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
impl MemorySnapshot {
    /// Create a new snapshot with current memory usage
    pub fn capture() -> Self {
        let rss = get_rss_bytes().unwrap_or(0);
        Self {
            rss_bytes: rss,
            peak_rss_bytes: rss,
        }
    }

    /// Get RSS in megabytes
    pub fn rss_mb(&self) -> f64 {
        self.rss_bytes as f64 / 1_048_576.0
    }

    /// Get peak RSS in megabytes
    pub fn peak_rss_mb(&self) -> f64 {
        self.peak_rss_bytes as f64 / 1_048_576.0
    }
}

/// Memory profile results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
pub struct MemoryProfile {
    /// Snapshot before operation
    pub before: MemorySnapshot,
    /// Snapshot after operation
    pub after: MemorySnapshot,
    /// Peak during operation
    pub peak: MemorySnapshot,
    /// Delta (after - before) in bytes
    pub delta_bytes: i64,
    /// Number of iterations profiled
    pub iterations: u64,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
impl MemoryProfile {
    /// Create from before/after snapshots
    pub fn from_snapshots(
        before: MemorySnapshot,
        after: MemorySnapshot,
        peak: MemorySnapshot,
        iterations: u64,
    ) -> Self {
        let delta = after.rss_bytes as i64 - before.rss_bytes as i64;
        Self {
            before,
            after,
            peak,
            delta_bytes: delta,
            iterations,
        }
    }

    /// Get delta in megabytes
    pub fn delta_mb(&self) -> f64 {
        self.delta_bytes as f64 / 1_048_576.0
    }

    /// Get peak RSS in megabytes
    pub fn peak_rss_mb(&self) -> f64 {
        self.peak.rss_mb()
    }

    /// Format as human-readable string
    pub fn format(&self) -> String {
        format!(
            "Memory Profile\n\
             {}\n\
             Before RSS:         {:.2}MB\n\
             After RSS:          {:.2}MB\n\
             Peak RSS:           {:.2}MB\n\
             Delta:              {:+.2}MB\n\
             Iterations:         {}\n",
            "-".repeat(40),
            self.before.rss_mb(),
            self.after.rss_mb(),
            self.peak.rss_mb(),
            self.delta_mb(),
            self.iterations
        )
    }
}

// ============================================================================
// Flamegraph Types
// ============================================================================

/// Flamegraph data for SVG generation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
pub struct FlamegraphData {
    /// Folded stack format (for inferno)
    pub folded_stacks: Vec<String>,
    /// Total samples
    pub sample_count: u64,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
impl FlamegraphData {
    /// Create new flamegraph data
    pub fn new() -> Self {
        Self {
            folded_stacks: Vec::new(),
            sample_count: 0,
        }
    }

    /// Add a folded stack sample
    pub fn add_stack(&mut self, stack: String) {
        self.folded_stacks.push(stack);
        self.sample_count += 1;
    }

    /// Check if there's data to generate
    pub fn has_data(&self) -> bool {
        !self.folded_stacks.is_empty()
    }
}

// ============================================================================
// Combined Profile Result
// ============================================================================

/// Complete profiling result combining all dimensions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
pub struct ProfileResult {
    /// Name of the profiled operation
    pub name: String,
    /// Timestamp when profiling started
    pub started_at: String,
    /// Timestamp when profiling ended
    pub ended_at: String,
    /// Total duration in milliseconds
    pub duration_ms: f64,
    /// Phase breakdown (if enabled)
    pub phase_breakdown: Option<PhaseBreakdown>,
    /// GIL contention analysis (if enabled)
    pub gil_analysis: Option<GilContentionResult>,
    /// Memory profile (if enabled)
    pub memory_profile: Option<MemoryProfile>,
    /// Flamegraph data (if enabled)
    pub flamegraph: Option<FlamegraphData>,
    /// Whether profiling succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
impl ProfileResult {
    /// Create a new successful result
    pub fn new(name: String) -> Self {
        Self {
            name,
            started_at: chrono::Utc::now().to_rfc3339(),
            ended_at: String::new(),
            duration_ms: 0.0,
            phase_breakdown: None,
            gil_analysis: None,
            memory_profile: None,
            flamegraph: None,
            success: true,
            error: None,
        }
    }

    /// Create a failed result
    pub fn failed(name: String, error: String) -> Self {
        Self {
            name,
            started_at: chrono::Utc::now().to_rfc3339(),
            ended_at: chrono::Utc::now().to_rfc3339(),
            duration_ms: 0.0,
            phase_breakdown: None,
            gil_analysis: None,
            memory_profile: None,
            flamegraph: None,
            success: false,
            error: Some(error),
        }
    }

    /// Set the phase breakdown
    pub fn with_phase_breakdown(mut self, breakdown: PhaseBreakdown) -> Self {
        self.phase_breakdown = Some(breakdown);
        self
    }

    /// Set the GIL analysis
    pub fn with_gil_analysis(mut self, analysis: GilContentionResult) -> Self {
        self.gil_analysis = Some(analysis);
        self
    }

    /// Set the memory profile
    pub fn with_memory_profile(mut self, profile: MemoryProfile) -> Self {
        self.memory_profile = Some(profile);
        self
    }

    /// Set the flamegraph data
    pub fn with_flamegraph(mut self, data: FlamegraphData) -> Self {
        self.flamegraph = Some(data);
        self
    }

    /// Finalize the result with duration
    pub fn finalize(mut self, duration: Duration) -> Self {
        self.ended_at = chrono::Utc::now().to_rfc3339();
        self.duration_ms = duration.as_secs_f64() * 1000.0;
        self
    }

    /// Format as human-readable string
    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("=== Profile: {} ===\n\n", self.name));

        if !self.success {
            output.push_str(&format!(
                "ERROR: {}\n",
                self.error.as_deref().unwrap_or("Unknown")
            ));
            return output;
        }

        output.push_str(&format!("Duration: {:.2}ms\n\n", self.duration_ms));

        if let Some(ref pb) = self.phase_breakdown {
            output.push_str(&pb.format());
            output.push('\n');
        }

        if let Some(ref ga) = self.gil_analysis {
            output.push_str(&ga.format());
            output.push('\n');
        }

        if let Some(ref mp) = self.memory_profile {
            output.push_str(&mp.format());
            output.push('\n');
        }

        if let Some(ref fg) = self.flamegraph {
            output.push_str(&format!(
                "Flamegraph: {} samples collected\n",
                fg.sample_count
            ));
        }

        output
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

// ============================================================================
// Profiler Configuration
// ============================================================================

/// What to profile
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
pub struct ProfileConfig {
    /// Enable phase breakdown timing
    pub enable_phase_breakdown: bool,
    /// Enable GIL contention analysis
    pub enable_gil_analysis: bool,
    /// Enable memory profiling
    pub enable_memory_profile: bool,
    /// Enable flamegraph generation
    pub enable_flamegraph: bool,
    /// Number of iterations for statistical significance
    pub iterations: u32,
    /// Warmup iterations
    pub warmup: u32,
    /// GIL test configuration
    pub gil_config: GilTestConfig,
    /// Output directory for flamegraph SVG
    pub output_dir: Option<String>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
impl ProfileConfig {
    /// Create config with all profiling enabled
    pub fn full() -> Self {
        Self {
            enable_phase_breakdown: true,
            enable_gil_analysis: true,
            enable_memory_profile: true,
            enable_flamegraph: true,
            iterations: 100,
            warmup: 10,
            gil_config: GilTestConfig::default(),
            output_dir: Some("./profile_output".to_string()),
        }
    }

    /// Create config for quick profiling (phase breakdown only)
    pub fn quick() -> Self {
        Self {
            enable_phase_breakdown: true,
            enable_gil_analysis: false,
            enable_memory_profile: false,
            enable_flamegraph: false,
            iterations: 20,
            warmup: 3,
            gil_config: GilTestConfig::default(),
            output_dir: None,
        }
    }

    /// Create config with custom settings
    pub fn custom(
        enable_phase_breakdown: bool,
        enable_gil_analysis: bool,
        enable_memory_profile: bool,
        enable_flamegraph: bool,
        iterations: u32,
        warmup: u32,
    ) -> Self {
        Self {
            enable_phase_breakdown,
            enable_gil_analysis,
            enable_memory_profile,
            enable_flamegraph,
            iterations,
            warmup,
            gil_config: GilTestConfig::default(),
            output_dir: None,
        }
    }

    /// Set GIL test configuration
    pub fn with_gil_config(mut self, config: GilTestConfig) -> Self {
        self.gil_config = config;
        self.enable_gil_analysis = true;
        self
    }

    /// Set output directory
    pub fn with_output_dir(mut self, dir: String) -> Self {
        self.output_dir = Some(dir);
        self
    }
}

// ============================================================================
// Profiler Implementation
// ============================================================================

/// Profiler for collecting timing and memory data
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
pub struct Profiler {
    config: ProfileConfig,
    start_time: Option<Instant>,
    phase_times: HashMap<String, Vec<u64>>,
    memory_before: Option<MemorySnapshot>,
    memory_peak: Option<MemorySnapshot>,
    flamegraph_data: FlamegraphData,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
impl Profiler {
    /// Create a new profiler with configuration
    pub fn new(config: ProfileConfig) -> Self {
        Self {
            config,
            start_time: None,
            phase_times: HashMap::new(),
            memory_before: None,
            memory_peak: None,
            flamegraph_data: FlamegraphData::new(),
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &ProfileConfig {
        &self.config
    }

    /// Start profiling
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        if self.config.enable_memory_profile {
            self.memory_before = Some(MemorySnapshot::capture());
            self.memory_peak = self.memory_before.clone();
        }
    }

    /// Record a phase timing
    pub fn record_phase(&mut self, phase: &str, duration_ns: u64) {
        self.phase_times
            .entry(phase.to_string())
            .or_default()
            .push(duration_ns);
    }

    /// Update peak memory if needed
    pub fn update_peak_memory(&mut self) {
        if self.config.enable_memory_profile {
            let current = MemorySnapshot::capture();
            if let Some(ref mut peak) = self.memory_peak {
                if current.rss_bytes > peak.rss_bytes {
                    peak.rss_bytes = current.rss_bytes;
                    peak.peak_rss_bytes = current.rss_bytes;
                }
            }
        }
    }

    /// Add a flamegraph stack sample
    pub fn add_stack(&mut self, stack: String) {
        if self.config.enable_flamegraph {
            self.flamegraph_data.add_stack(stack);
        }
    }

    /// Finalize and generate profile result
    pub fn finalize(self, name: String, operation_count: u64) -> ProfileResult {
        let duration = self
            .start_time
            .map(|s| s.elapsed())
            .unwrap_or(Duration::ZERO);

        let mut result = ProfileResult::new(name);

        // Phase breakdown
        if self.config.enable_phase_breakdown && !self.phase_times.is_empty() {
            let breakdown = PhaseBreakdown::from_times(
                self.phase_times,
                operation_count,
                duration.as_nanos() as u64,
            );
            result = result.with_phase_breakdown(breakdown);
        }

        // Memory profile
        if self.config.enable_memory_profile {
            if let (Some(before), Some(peak)) = (self.memory_before, self.memory_peak) {
                let after = MemorySnapshot::capture();
                let profile = MemoryProfile::from_snapshots(before, after, peak, operation_count);
                result = result.with_memory_profile(profile);
            }
        }

        // Flamegraph data
        if self.config.enable_flamegraph && self.flamegraph_data.has_data() {
            result = result.with_flamegraph(self.flamegraph_data);
        }

        result.finalize(duration)
    }
}

// ============================================================================
// Flamegraph Generation
// ============================================================================

/// Generate SVG flamegraph from folded stacks
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
pub fn generate_flamegraph_svg(
    folded_stacks: &[String],
    title: &str,
    output_path: &str,
) -> Result<(), String> {
    use std::fs::File;

    if folded_stacks.is_empty() {
        return Err("No stacks to generate flamegraph from".to_string());
    }

    // Create output file
    let file =
        File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = BufWriter::new(file);

    // Configure flamegraph options
    let mut opts = inferno::flamegraph::Options::default();
    opts.title = title.to_string();
    opts.colors =
        inferno::flamegraph::color::Palette::Basic(inferno::flamegraph::color::BasicPalette::Aqua);
    opts.count_name = "samples".to_string();

    // Join stacks into input format
    let input = folded_stacks.join("\n");
    let input_bytes = input.as_bytes();

    // Generate flamegraph
    inferno::flamegraph::from_reader(&mut opts, input_bytes, &mut writer)
        .map_err(|e| format!("Failed to generate flamegraph: {}", e))?;

    writer
        .flush()
        .map_err(|e| format!("Failed to flush output: {}", e))?;

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get RSS bytes (macOS and Linux compatible)
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-performance-profiler-rs.md#source
pub fn get_rss_bytes() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        // macOS reports ru_maxrss in bytes. This is peak RSS rather than current
        // RSS, but it avoids shelling out to `ps`, which is often blocked in
        // test sandboxes.
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
        let rc = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
        if rc != 0 {
            return None;
        }
        let usage = unsafe { usage.assume_init() };
        u64::try_from(usage.ru_maxrss).ok().filter(|rss| *rss > 0)
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, read from /proc/self/statm
        use std::fs;
        let statm = fs::read_to_string("/proc/self/statm").ok()?;
        let parts: Vec<&str> = statm.split_whitespace().collect();
        let rss_pages: u64 = parts.get(1)?.parse().ok()?;
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 };
        Some(rss_pages * page_size)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_timing() {
        let mut timing = PhaseTiming::new();
        timing.record(1_000_000); // 1ms
        timing.record(2_000_000); // 2ms
        timing.record(3_000_000); // 3ms

        assert_eq!(timing.count, 3);
        assert_eq!(timing.total_ns, 6_000_000);
        assert_eq!(timing.min_ns, 1_000_000);
        assert_eq!(timing.max_ns, 3_000_000);
        assert!((timing.avg_ms() - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_phase_breakdown() {
        let mut times = HashMap::new();
        times.insert(
            "PythonExtract".to_string(),
            vec![1_000_000, 1_000_000, 1_000_000],
        );
        times.insert(
            "RustConvert".to_string(),
            vec![2_000_000, 2_000_000, 2_000_000],
        );

        let breakdown = PhaseBreakdown::from_times(times, 3, 9_000_000);

        assert_eq!(breakdown.operation_count, 3);
        assert_eq!(breakdown.phase_names().len(), 2);

        let percentages = breakdown.percentage_breakdown();
        assert!((percentages["PythonExtract"] - 33.33).abs() < 1.0);
        assert!((percentages["RustConvert"] - 66.67).abs() < 1.0);
    }

    #[test]
    fn test_gil_contention_result() {
        let config = GilTestConfig::with_workers(4);
        let result = GilContentionResult::from_measurements(
            config,
            1000.0, // sequential
            300.0,  // concurrent (should be ~250 for perfect parallelism)
            vec![280.0, 290.0, 295.0, 300.0],
        );

        assert!(result.actual_speedup > 3.0);
        assert!(result.efficiency_percent > 75.0);
        assert!(result.gil_release_effective);
    }

    #[test]
    fn test_profile_config() {
        let full = ProfileConfig::full();
        assert!(full.enable_phase_breakdown);
        assert!(full.enable_gil_analysis);
        assert!(full.enable_memory_profile);
        assert!(full.enable_flamegraph);
        assert_eq!(full.iterations, 100);

        let quick = ProfileConfig::quick();
        assert!(quick.enable_phase_breakdown);
        assert!(!quick.enable_gil_analysis);
        assert_eq!(quick.iterations, 20);
    }

    #[test]
    fn test_profile_result() {
        let result = ProfileResult::new("test_op".to_string()).finalize(Duration::from_millis(100));

        assert_eq!(result.name, "test_op");
        assert!(result.success);
        assert!((result.duration_ms - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_memory_snapshot() {
        let snapshot = MemorySnapshot::capture();
        // Just verify it doesn't panic and returns something reasonable
        assert!(snapshot.rss_bytes > 0 || cfg!(not(any(target_os = "macos", target_os = "linux"))));
    }
}
// CODEGEN-END
