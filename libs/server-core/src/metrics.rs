/// Hook surface for connection-oriented runtimes.
/// @spec projects/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
pub trait ConnectionMetrics: Send + Sync + 'static {
    fn connection_accepted(&self) {}
    fn connection_rejected(&self) {}
    fn connection_closed(&self) {}
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopConnectionMetrics;

impl ConnectionMetrics for NoopConnectionMetrics {}
