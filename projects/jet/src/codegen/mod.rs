// SPEC-MANAGED: .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
// HANDWRITE-BEGIN
//! `jet codegen` — generate frontend code from API specs.
//!
//! Currently supports `jet codegen openapi`: read an OpenAPI 3.0/3.1 document
//! and emit TypeScript types, a typed fetch client, and TanStack Query hooks.
//! The pipeline is: parse → normalize 3.0/3.1 → resolve `$ref`s → emit
//! `types.ts` / `runtime.ts` / `client.ts` / `hooks.ts` / `index.ts`.

use anyhow::{Context, Result};
use std::path::PathBuf;

pub mod client_emit;
pub mod hooks_emit;
pub mod names;
pub mod openapi;
pub mod plan;
pub mod tsmap;
pub mod types_emit;

use openapi::Spec;
use tsmap::TypeMap;

/// What the generator emits, selected by CLI flags.
///
/// @spec .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
#[derive(Debug, Clone)]
pub struct GenOptions {
    pub spec_path: PathBuf,
    pub out_dir: PathBuf,
    pub client_name: String,
    pub emit_types: bool,
    pub emit_client: bool,
    pub emit_hooks: bool,
}

/// A single generated file, relative to the output directory.
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub rel_path: String,
    pub contents: String,
}

/// The full in-memory generation result (so tests can assert without I/O).
#[derive(Debug, Clone, Default)]
pub struct GeneratedOutput {
    pub files: Vec<GeneratedFile>,
}

/// Pure core: spec JSON text → generated files. No filesystem access.
///
/// @spec .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
pub fn generate(spec_json: &str, opts: &GenOptions) -> Result<GeneratedOutput> {
    let spec: Spec = serde_json::from_str(spec_json).context("failed to parse OpenAPI spec")?;
    let tm = build_type_map(&spec);
    let plans = plan::build(&spec, &tm);

    let mut files = Vec::new();
    if opts.emit_types {
        files.push(GeneratedFile {
            rel_path: "types.ts".to_string(),
            contents: types_emit::emit(&spec, &tm),
        });
    }
    if opts.emit_client {
        files.push(GeneratedFile {
            rel_path: "runtime.ts".to_string(),
            contents: client_emit::emit_runtime(),
        });
        files.push(GeneratedFile {
            rel_path: "client.ts".to_string(),
            contents: client_emit::emit_client(&plans, &tm, opts),
        });
    }
    if opts.emit_hooks {
        files.push(GeneratedFile {
            rel_path: "hooks.ts".to_string(),
            contents: hooks_emit::emit(&plans, &tm),
        });
    }
    files.push(GeneratedFile {
        rel_path: "index.ts".to_string(),
        contents: emit_index(opts),
    });
    Ok(GeneratedOutput { files })
}

/// CLI entry: read spec, generate, write files. Returns a process exit code
/// (0 ok, 1 generation/write error, 2 spec read error).
///
/// @spec .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
pub fn run(opts: &GenOptions) -> i32 {
    let spec_json = match std::fs::read_to_string(&opts.spec_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("jet codegen: cannot read {}: {e}", opts.spec_path.display());
            return 2;
        }
    };
    let output = match generate(&spec_json, opts) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("jet codegen: {e:#}");
            return 1;
        }
    };
    if let Err(e) = std::fs::create_dir_all(&opts.out_dir) {
        eprintln!("jet codegen: cannot create {}: {e}", opts.out_dir.display());
        return 1;
    }
    for file in &output.files {
        let path = opts.out_dir.join(&file.rel_path);
        if let Err(e) = std::fs::write(&path, &file.contents) {
            eprintln!("jet codegen: cannot write {}: {e}", path.display());
            return 1;
        }
        println!("generated {}", path.display());
    }
    0
}

/// Assign a deterministic, collision-free TypeScript type name to each
/// component schema key.
pub fn build_type_map(spec: &Spec) -> TypeMap {
    let mut reg = names::NameRegistry::new();
    let mut map = std::collections::BTreeMap::new();
    for key in spec.components.schemas.keys() {
        let name = reg.unique(&names::to_pascal(key));
        map.insert(key.clone(), name);
    }
    TypeMap { names: map }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn full_opts() -> GenOptions {
        GenOptions {
            spec_path: PathBuf::new(),
            out_dir: PathBuf::new(),
            client_name: "createClient".to_string(),
            emit_types: true,
            emit_client: true,
            emit_hooks: true,
        }
    }

    const MINIMAL: &str = r##"{
      "openapi": "3.0.0",
      "info": { "title": "Mini", "version": "1.0.0" },
      "paths": {
        "/pets": {
          "get": {
            "operationId": "listPets",
            "responses": { "200": { "content": { "application/json": {
              "schema": { "type": "array", "items": { "$ref": "#/components/schemas/Pet" } } } } } }
          }
        }
      },
      "components": { "schemas": {
        "Pet": { "type": "object", "properties": { "id": { "type": "integer" }, "name": { "type": "string" } }, "required": ["id", "name"] }
      } }
    }"##;

    #[test]
    fn generates_all_files() {
        let out = generate(MINIMAL, &full_opts()).unwrap();
        let names: Vec<&str> = out.files.iter().map(|f| f.rel_path.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "types.ts",
                "runtime.ts",
                "client.ts",
                "hooks.ts",
                "index.ts"
            ]
        );
    }

    #[test]
    fn types_only_skips_client_and_hooks() {
        let mut opts = full_opts();
        opts.emit_client = false;
        opts.emit_hooks = false;
        let out = generate(MINIMAL, &opts).unwrap();
        let names: Vec<&str> = out.files.iter().map(|f| f.rel_path.as_str()).collect();
        assert_eq!(names, vec!["types.ts", "index.ts"]);
    }

    #[test]
    fn deterministic_across_runs() {
        let a = generate(MINIMAL, &full_opts()).unwrap();
        let b = generate(MINIMAL, &full_opts()).unwrap();
        for (fa, fb) in a.files.iter().zip(b.files.iter()) {
            assert_eq!(fa.rel_path, fb.rel_path);
            assert_eq!(fa.contents, fb.contents);
        }
    }

    #[test]
    fn invalid_spec_is_an_error() {
        assert!(generate("{ not json", &full_opts()).is_err());
    }

    #[test]
    fn custom_client_name() {
        let mut opts = full_opts();
        opts.client_name = "makeApi".to_string();
        let out = generate(MINIMAL, &opts).unwrap();
        let client = out
            .files
            .iter()
            .find(|f| f.rel_path == "client.ts")
            .unwrap();
        assert!(client
            .contents
            .contains("export function makeApi(config: ClientConfig)"));
        assert!(client.contents.contains("ReturnType<typeof makeApi>"));
    }
}
// HANDWRITE-END
