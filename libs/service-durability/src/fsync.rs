// SPEC-MANAGED: libs/service-durability/tech-design/semantic/source/libs-service-durability-src-fsync-rs.md#rust-source-unit
// CODEGEN-BEGIN
/// Shared flush policy for local durable files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// @spec libs/service-durability/tech-design/semantic/source/libs-service-durability-src-fsync-rs.md#source
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

/// @spec libs/service-durability/tech-design/semantic/source/libs-service-durability-src-fsync-rs.md#source
impl FsyncPolicy {
    pub fn should_sync_immediately(self) -> bool {
        matches!(self, Self::Always)
    }
}
// CODEGEN-END
