// CODEGEN-BEGIN
//! How much durability the caller is asking for -- and which of the four
//! answers this crate acts on itself.
//!
//! The variants are not four points on one scale, because the two consumers in
//! this crate read them differently and that asymmetry is the actual contract:
//!
//! - [`atomic::atomic_write`](crate::atomic_write) fsyncs the temp file and the
//!   parent directory for every policy **except** [`FsyncPolicy::Os`]. So a
//!   snapshot written under `EverySec` or `Interval` is fully durable on return;
//!   the batching those names promise does not apply to atomic replacement.
//! - [`FramedLogWriter::maybe_sync`](crate::FramedLogWriter::maybe_sync) acts on
//!   `EverySec` and on nothing else. Under `Interval` it returns `Ok(())`
//!   without syncing, so on the append path nothing in this crate will ever
//!   fsync an `Interval` log -- the storage owner must call `sync()` itself.
//!
//! Which leaves `Interval` and `Os` behaviourally identical for a caller that
//! appends and never calls `sync()`: the difference is stated intent, not
//! observable durability. `should_sync_immediately` is true for `Always` alone,
//! and `Always` is the `Default`.
/// Shared flush policy for local durable files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FsyncPolicy {
    /// Flush and fsync at the durability boundary.
    #[default]
    Always,
    /// Flush writes and let the caller batch fsyncs on a timer.
    EverySec,
    /// Flush writes and let a higher-level storage owner decide the interval.
    Interval,
    /// Rely on the OS page cache.
    Os,
}

impl FsyncPolicy {
    pub fn should_sync_immediately(self) -> bool {
        matches!(self, Self::Always)
    }
}
// CODEGEN-END
