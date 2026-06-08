//! Task executors for different execution environments

#[cfg(feature = "k8s")]
pub mod k8s;

#[cfg(feature = "k8s")]
pub use k8s::{K8sJobExecutor, K8sJobExecutorConfig, OffloadedJobInfo};
