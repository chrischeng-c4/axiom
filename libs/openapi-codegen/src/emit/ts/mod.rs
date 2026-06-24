//! TypeScript emitter: read an OpenAPI 3.0/3.1 document and emit TS types, a
//! typed fetch/axios client, and TanStack Query hooks.
//!
//! Pipeline: parse → normalize 3.0/3.1 → resolve `$ref`s → emit `types.ts` /
//! `runtime.ts` / `client.ts` / `hooks.ts` / `index.ts`.

pub mod client_emit;
pub mod hooks_emit;
pub mod plan;
pub mod tsmap;
pub mod types_emit;

use crate::ir::build_type_map;
use crate::ir::openapi::Spec;
use crate::{GenOptions, GeneratedFile, GeneratedOutput};
use anyhow::{Context, Result};

/// Pure TS generation: spec JSON text → in-memory files. No filesystem access.
pub fn generate(spec_json: &str, opts: &GenOptions) -> Result<GeneratedOutput> {
    let spec: Spec = serde_json::from_str(spec_json).context("failed to parse OpenAPI spec")?;
    let tm = build_type_map(&spec);
    let plans = plan::build(&spec, &tm);

    let mut files = Vec::new();
    if opts.emit_types {
        files.push(GeneratedFile {
            rel_path: "types.ts".to_string(),
            contents: types_emit::emit(&spec, &tm, &plans),
        });
    }
    if opts.emit_client {
        files.push(GeneratedFile {
            rel_path: "runtime.ts".to_string(),
            contents: client_emit::emit_runtime(opts.http_client),
        });
        files.push(GeneratedFile {
            rel_path: "client.ts".to_string(),
            contents: client_emit::emit_client(&plans, opts),
        });
    }
    if opts.emit_hooks {
        files.push(GeneratedFile {
            rel_path: "hooks.ts".to_string(),
            contents: hooks_emit::emit(&plans),
        });
    }
    files.push(GeneratedFile {
        rel_path: "index.ts".to_string(),
        contents: emit_index(opts),
    });
    Ok(GeneratedOutput { files })
}

fn emit_index(opts: &GenOptions) -> String {
    let mut out = String::from(types_emit::HEADER);
    if opts.emit_types {
        out.push_str("export * from \"./types\";\n");
    }
    if opts.emit_client {
        out.push_str("export * from \"./runtime\";\n");
        out.push_str("export * from \"./client\";\n");
    }
    if opts.emit_hooks {
        out.push_str("export * from \"./hooks\";\n");
    }
    out
}
