//! `httpkit-demo` — consumer of `mambalibs.http` showing the pydantic-like BaseModel
//! pattern for user-defined request payload models.
//!
//! Every module in this crate is produced by TD v2 codegen from specs
//! under `.aw/tech-design/projects/httpkit-demo/`. The `pub mod`
//! declarations below and the `register()` body are maintained by
//! apply.rs's `auto_wire_mamba_lib` post-pass.

use cclab_mamba_registry::{MambaModule, ModuleRegistrar, MAMBA_MODULES};
use linkme::distributed_slice;

pub struct HttpkitDemoModule;

impl MambaModule for HttpkitDemoModule {
    fn name(&self) -> &'static str {
        "mambalibs.httpkit_demo"
    }

    fn doc(&self) -> &'static str {
        "Httpkit demo crate exercising the pydantic-like BaseModel pattern"
    }

    fn register(&self, r: &mut ModuleRegistrar) {
        // SPEC-MANAGED: generated/mamba-registry#mamba-register-body
        // CODEGEN-BEGIN
        create_user_request::register(r);
        // CODEGEN-END
    }
}

#[distributed_slice(MAMBA_MODULES)]
static HTTPKIT_DEMO_MODULE: &dyn MambaModule = &HttpkitDemoModule;
// SPEC-MANAGED: generated/mamba-registry#mamba-mod-decls
// CODEGEN-BEGIN
pub mod create_user_request;
// CODEGEN-END
