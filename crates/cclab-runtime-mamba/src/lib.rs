//! Mamba binding for the cclab async runtime.
//!
//! Provides `serve`, `spawn`, `sleep`, and `gather` primitives backed by a
//! module-global Tokio runtime. The HTTP server (`serve`) reads the minimal
//! [`types::MbRouter`] route table produced by the current Mamba HTTP surface.
//!
//! # Module name
//!
//! Import in Mamba as `cclab.runtime`:
//! ```python
//! from cclab.runtime import serve, sleep, spawn, gather
//! ```

pub mod methods;
pub mod types;

use cclab_mamba_registry::{rt_sym, MambaModule, ModuleRegistrar, MAMBA_MODULES};
use linkme::distributed_slice;

// ── RuntimeMambaModule ────────────────────────────────────────────────────────

/// The `cclab-runtime-mamba` native module descriptor.
pub struct RuntimeMambaModule;

impl MambaModule for RuntimeMambaModule {
    fn name(&self) -> &'static str {
        "cclab.runtime"
    }

    fn doc(&self) -> &'static str {
        "Mamba bindings for the cclab async runtime — serve, spawn, sleep, gather"
    }

    fn register(&self, r: &mut ModuleRegistrar) {
        use crate::methods::{
            mb_runtime_gather, mb_runtime_serve, mb_runtime_sleep, mb_runtime_spawn,
        };

        r.add_symbols([
            rt_sym!(
                "serve",
                mb_runtime_serve,
                "serve(router, host: str, port: int) -> handle"
            ),
            rt_sym!("spawn", mb_runtime_spawn, "spawn(coro) -> task"),
            rt_sym!("sleep", mb_runtime_sleep, "sleep(seconds: float) -> None"),
            rt_sym!("gather", mb_runtime_gather, "gather(coros: list) -> None"),
        ]);
    }
}

// ── Auto-registration ─────────────────────────────────────────────────────────

#[distributed_slice(MAMBA_MODULES)]
static RUNTIME_MAMBA_MODULE: &dyn MambaModule = &RuntimeMambaModule;
