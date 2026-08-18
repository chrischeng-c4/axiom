// CODEGEN-BEGIN
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
