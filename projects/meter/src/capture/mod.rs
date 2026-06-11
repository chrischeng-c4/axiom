// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-capture-mod-rs.md#source
// CODEGEN-BEGIN
//! Capture-mode populators (擷取) — observe a workload from the outside.
//!
//! Capture is the half of `meter` that runs/observes external processes (in
//! contrast to the in-process `embed`/埋点 probes in `performance`). It is gated
//! behind the `capture` feature so the engine rlib stays free of process-spawn
//! machinery for pure-library consumers.
//!
//! This wave ships [`delegate`] (the `meter test` delegate+forward path),
//! [`audit`] (the `meter audit` cargo-audit caller), [`bench`] (the `meter bench`
//! cargo-bench delegate + regression-baseline loader), the C1 profiling pair
//! [`sampler`] (spawn + platform stack sampler -> folded stacks) + [`fold`]
//! (folded stacks -> ranked `Hotspot` findings, the default stdout),
//! [`vitals`] (the meter.toml measurement contract + L1 vitals capture window),
//! and [`run`] (the composite `meter run` sweep that folds every sub-verb into
//! ONE worst-wins report).

pub mod audit;
pub mod bench;
pub mod delegate;
pub mod fold;
pub mod run;
pub mod sampler;
pub mod vitals;
// CODEGEN-END
