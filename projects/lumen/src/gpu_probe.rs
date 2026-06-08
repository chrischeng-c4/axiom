//! Pod-level GPU availability probe.
//!
//! `WgpuBruteForceIndex` already degrades gracefully per-field — if no
//! compatible adapter is present, it transparently runs `HnswCpuIndex`
//! instead. This module surfaces the same probe **once at startup** so:
//!
//! - operators see a clear "GPU available / not available" line in the
//!   pod log on boot, before any collection is created;
//! - the `lumen_gpu_available` Prometheus gauge reports 0/1 from the
//!   first scrape, so dashboards/alerts know whether vector-search
//!   queries on this pod will hit the WGSL kernel or the CPU
//!   fallback;
//! - downstream code can short-circuit `backend: wgpu-brute-force`
//!   schemas with a 400 at create time on no-GPU pods if the operator
//!   has set `LUMEN_REQUIRE_GPU=on`.
//!
//! The probe is feature-gated by `gpu`. Without that feature the probe
//! returns `Unavailable("gpu feature disabled at compile time")`.

use std::time::Duration;

#[derive(Debug, Clone)]
pub enum GpuStatus {
    Available {
        adapter_name: String,
        backend: String,
        device_type: String,
    },
    Unavailable(String),
}

impl GpuStatus {
    pub fn is_available(&self) -> bool {
        matches!(self, GpuStatus::Available { .. })
    }

    pub fn label(&self) -> &'static str {
        match self {
            GpuStatus::Available { .. } => "available",
            GpuStatus::Unavailable(_) => "unavailable",
        }
    }
}

/// Probe the host for a compatible GPU adapter and return what we
/// found. Never panics, never blocks longer than `timeout`.
#[cfg(feature = "gpu")]
pub fn probe(timeout: Duration) -> GpuStatus {
    let probe_fut = async {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await;
        match adapter {
            Some(a) => {
                let info = a.get_info();
                GpuStatus::Available {
                    adapter_name: info.name.clone(),
                    backend: format!("{:?}", info.backend),
                    device_type: format!("{:?}", info.device_type),
                }
            }
            None => GpuStatus::Unavailable("no compatible adapter".into()),
        }
    };

    match pollster::block_on(async { tokio::time::timeout(timeout, probe_fut).await }) {
        Ok(s) => s,
        Err(_) => GpuStatus::Unavailable(format!("probe timed out after {timeout:?}")),
    }
}

#[cfg(not(feature = "gpu"))]
pub fn probe(_timeout: Duration) -> GpuStatus {
    GpuStatus::Unavailable("gpu feature disabled at compile time".into())
}

/// Emit a single tracing line so operators see GPU status in the pod
/// log on boot. Returns the same status the caller can stash for the
/// `/metrics` gauge.
pub fn probe_and_log() -> GpuStatus {
    let status = probe(Duration::from_secs(5));
    match &status {
        GpuStatus::Available {
            adapter_name,
            backend,
            device_type,
        } => {
            tracing::info!(
                target: "lumen.gpu",
                event = "gpu_probe",
                available = true,
                adapter = %adapter_name,
                backend = %backend,
                device_type = %device_type,
                "GPU adapter detected — wgpu-brute-force vector backend is active"
            );
        }
        GpuStatus::Unavailable(reason) => {
            tracing::warn!(
                target: "lumen.gpu",
                event = "gpu_probe",
                available = false,
                reason = %reason,
                "no GPU adapter — vector fields declared `wgpu-brute-force` will run on the CPU HNSW fallback"
            );
        }
    }
    status
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_does_not_panic() {
        // CI runners typically lack a GPU. The probe must never crash
        // and must always return a `GpuStatus`.
        let s = probe(Duration::from_secs(2));
        let _ = s.is_available();
        let _ = s.label();
    }

    #[test]
    #[cfg(not(feature = "gpu"))]
    fn probe_without_feature_is_unavailable() {
        assert!(!probe(Duration::from_secs(1)).is_available());
    }
}
