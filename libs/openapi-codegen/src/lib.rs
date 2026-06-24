//! `openapi-codegen` — generate a typed API client from an OpenAPI 3.0/3.1
//! document, in TypeScript, Python, or Rust. Reusable polyglot codegen core,
//! extracted from `jet codegen openapi` so any CLI can compose it.
//!
//! Architecture: a language-neutral [`ir`] (document model, naming, type-name
//! map) feeds a per-language emitter under [`emit`]. The target language is
//! [`GenOptions::lang`]:
//! - [`Lang::Ts`]  → TypeScript: types + fetch/axios client + TanStack Query hooks
//! - [`Lang::Py`]  → Python: pydantic models + httpx client (planned)
//! - [`Lang::Rust`]→ Rust: serde models + reqwest client (planned)
//!
//! [`generate`] is the pure core (spec text → in-memory files, no I/O); [`run`]
//! is the filesystem-writing CLI entry.

use anyhow::Result;
use std::path::PathBuf;

pub mod emit;
pub mod ir;

pub use ir::{build_type_map, TypeMap};

/// Target language for the generated client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    /// TypeScript: types + a typed fetch/axios client + TanStack Query hooks.
    #[default]
    Ts,
    /// Python: pydantic models + an httpx client. (planned)
    Py,
    /// Rust: serde models + a reqwest client. (planned)
    Rust,
}

/// HTTP runtime backend for the generated TypeScript client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HttpClient {
    /// Native `fetch` (zero runtime dependency).
    #[default]
    Fetch,
    /// `axios` (peer dependency of the generated output).
    Axios,
}

/// What the generator emits, selected by CLI flags.
#[derive(Debug, Clone)]
pub struct GenOptions {
    /// Target language for the generated client.
    pub lang: Lang,
    pub spec_path: PathBuf,
    pub out_dir: PathBuf,
    pub client_name: String,
    pub http_client: HttpClient,
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

/// Pure core: spec JSON text → generated files. No filesystem access. Dispatches
/// to the per-language emitter selected by [`GenOptions::lang`].
pub fn generate(spec_json: &str, opts: &GenOptions) -> Result<GeneratedOutput> {
    match opts.lang {
        Lang::Ts => emit::ts::generate(spec_json, opts),
        Lang::Py => emit::py::generate(spec_json, opts),
        Lang::Rust => emit::rust::generate(spec_json, opts),
    }
}

/// CLI entry: read spec, generate, write files. Returns a process exit code
/// (0 ok, 1 generation/write error, 2 spec read error).
pub fn run(opts: &GenOptions) -> i32 {
    let spec_json = match std::fs::read_to_string(&opts.spec_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "openapi-codegen: cannot read {}: {e}",
                opts.spec_path.display()
            );
            return 2;
        }
    };
    let output = match generate(&spec_json, opts) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("openapi-codegen: {e:#}");
            return 1;
        }
    };
    if let Err(e) = std::fs::create_dir_all(&opts.out_dir) {
        eprintln!(
            "openapi-codegen: cannot create {}: {e}",
            opts.out_dir.display()
        );
        return 1;
    }
    for file in &output.files {
        let path = opts.out_dir.join(&file.rel_path);
        if let Err(e) = std::fs::write(&path, &file.contents) {
            eprintln!("openapi-codegen: cannot write {}: {e}", path.display());
            return 1;
        }
        println!("generated {}", path.display());
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_opts() -> GenOptions {
        GenOptions {
            lang: Lang::Ts,
            spec_path: PathBuf::new(),
            out_dir: PathBuf::new(),
            client_name: "createClient".to_string(),
            http_client: HttpClient::Fetch,
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
    fn every_lang_generates_non_empty() {
        for lang in [Lang::Ts, Lang::Py, Lang::Rust] {
            let mut opts = full_opts();
            opts.lang = lang;
            let out = generate(MINIMAL, &opts).expect("emitter runs");
            assert!(!out.files.is_empty(), "{lang:?} produced no files");
        }
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

    fn content<'a>(out: &'a GeneratedOutput, name: &str) -> &'a str {
        out.files
            .iter()
            .find(|f| f.rel_path == name)
            .unwrap()
            .contents
            .as_str()
    }

    #[test]
    fn http_backend_only_changes_runtime() {
        let fetch = generate(MINIMAL, &full_opts()).unwrap();
        let mut axios_opts = full_opts();
        axios_opts.http_client = HttpClient::Axios;
        let axios = generate(MINIMAL, &axios_opts).unwrap();

        // Everything except runtime.ts is byte-identical across backends.
        for name in ["types.ts", "client.ts", "hooks.ts", "index.ts"] {
            assert_eq!(
                content(&fetch, name),
                content(&axios, name),
                "{name} differs across backends"
            );
        }

        // The fetch runtime uses native fetch; the axios runtime imports axios.
        let fetch_rt = content(&fetch, "runtime.ts");
        assert!(fetch_rt.contains("const doFetch = config.fetch ?? fetch;"));
        assert!(!fetch_rt.contains("axios"));
        let axios_rt = content(&axios, "runtime.ts");
        assert!(axios_rt.contains("import axios from \"axios\";"));
        assert!(axios_rt.contains("axios?: AxiosInstance;"));
        assert!(axios_rt.contains("config.axios ?? axios.create()"));
    }
}
