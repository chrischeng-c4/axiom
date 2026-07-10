// SPEC-MANAGED: libs/openapi-codegen/tech-design/semantic/source/libs-openapi-codegen-src-emit-rust-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Rust emitter: read an OpenAPI 3.0/3.1/3.2 document and emit serde models
//! plus a typed `reqwest::blocking` client.
//!
//! Pipeline: parse → `models.rs` (serde struct/alias per component schema) +
//! `client.rs` (one `Client` method per operation) + `mod.rs`.
//!
//! OpenAPI 3.2 `query` operations (HTTP QUERY, RFC 10008) emit a client
//! method that dispatches via
//! `reqwest::Method::from_bytes(b"QUERY")` — `reqwest::blocking::Client` has
//! no dedicated `.query()` verb method (that name is the querystring
//! builder). `Client::with_post_fallback(true)` flips that dispatch to
//! `POST` against the operation's documented twin path at request time.

pub mod client_emit;
pub mod models_emit;
pub mod rsmap;

use crate::ir::build_type_map;
use crate::ir::openapi::Spec;
use crate::ir::operations;
use crate::{GenOptions, GeneratedFile, GeneratedOutput};
use anyhow::{Context, Result};

/// Pure Rust generation: spec JSON text → in-memory files. No filesystem access.
/// @spec libs/openapi-codegen/tech-design/semantic/source/libs-openapi-codegen-src-emit-rust-mod-rs.md#source
pub fn generate(spec_json: &str, opts: &GenOptions) -> Result<GeneratedOutput> {
    let spec: Spec = serde_json::from_str(spec_json).context("failed to parse OpenAPI spec")?;
    let tm = build_type_map(&spec);
    let ops = operations::build(&spec);

    let mut files = Vec::new();
    if opts.emit_types {
        files.push(GeneratedFile {
            rel_path: "models.rs".to_string(),
            contents: models_emit::emit(&spec, &tm),
        });
    }
    if opts.emit_client {
        files.push(GeneratedFile {
            rel_path: "client.rs".to_string(),
            contents: client_emit::emit(&ops, &tm),
        });
    }
    files.push(GeneratedFile {
        rel_path: "mod.rs".to_string(),
        contents: emit_mod(opts),
    });
    Ok(GeneratedOutput { files })
}

fn emit_mod(opts: &GenOptions) -> String {
    let mut out = String::from(models_emit::HEADER);
    if opts.emit_types {
        out.push_str("pub mod models;\n");
    }
    if opts.emit_client {
        out.push_str("pub mod client;\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HttpClient, Lang};
    use std::path::PathBuf;

    const SPEC: &str = r##"{
      "openapi": "3.0.0",
      "info": { "title": "Mini", "version": "1.0.0" },
      "paths": {
        "/pets/{petId}": {
          "get": {
            "operationId": "getPetById",
            "parameters": [{ "name": "petId", "in": "path", "required": true, "schema": { "type": "integer" } }],
            "responses": { "200": { "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Pet" } } } } }
          }
        }
      },
      "components": { "schemas": {
        "Pet": { "type": "object", "properties": { "id": { "type": "integer" }, "name": { "type": "string" }, "type": { "type": "string" } }, "required": ["id", "name"] }
      } }
    }"##;

    /// OpenAPI 3.2 fixture: a `query` operation (RFC 10008 HTTP QUERY) with a
    /// sibling `post` twin on the same path — the default POST-twin fallback
    /// convention (epic #1296).
    const SPEC_32_QUERY: &str = r##"{
      "openapi": "3.2.0",
      "info": { "title": "Mini", "version": "1.0.0" },
      "paths": {
        "/pets": {
          "query": {
            "operationId": "searchPets",
            "requestBody": { "required": true, "content": { "application/json": { "schema": { "type": "object" } } } },
            "responses": { "200": { "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Pet" } } } } }
          },
          "post": {
            "operationId": "createPet",
            "requestBody": { "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Pet" } } } },
            "responses": { "201": { "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Pet" } } } } }
          }
        }
      },
      "components": { "schemas": {
        "Pet": { "type": "object", "properties": { "id": { "type": "integer" }, "name": { "type": "string" } }, "required": ["id", "name"] }
      } }
    }"##;

    fn opts() -> GenOptions {
        GenOptions {
            lang: Lang::Rust,
            spec_path: PathBuf::new(),
            out_dir: PathBuf::new(),
            client_name: "Client".to_string(),
            http_client: HttpClient::Fetch,
            emit_types: true,
            emit_client: true,
            emit_hooks: false,
        }
    }

    fn file<'a>(out: &'a GeneratedOutput, name: &str) -> &'a str {
        out.files
            .iter()
            .find(|f| f.rel_path == name)
            .unwrap()
            .contents
            .as_str()
    }

    #[test]
    fn emits_models_client_mod() {
        let out = generate(SPEC, &opts()).unwrap();
        let names: Vec<&str> = out.files.iter().map(|f| f.rel_path.as_str()).collect();
        assert_eq!(names, vec!["models.rs", "client.rs", "mod.rs"]);
    }

    #[test]
    fn serde_struct_typed_with_rename_and_skip() {
        let out = generate(SPEC, &opts()).unwrap();
        let models = file(&out, "models.rs");
        assert!(models.contains("use serde::{Deserialize, Serialize};"));
        assert!(models.contains("#[derive(Debug, Clone, Serialize, Deserialize)]"));
        assert!(models.contains("pub struct Pet {"));
        assert!(models.contains("    pub id: i64,\n"));
        assert!(models.contains("    pub name: String,\n"));
        // `type` is a keyword → renamed field + serde rename, and it is optional.
        assert!(models.contains("#[serde(rename = \"type\")]"));
        assert!(models.contains("pub type_: Option<String>,"));
    }

    #[test]
    fn reqwest_client_method_typed() {
        let out = generate(SPEC, &opts()).unwrap();
        let client = file(&out, "client.rs");
        assert!(client.contains("pub struct TransportPolicy"));
        assert!(client.contains("const DEFAULT_MAX_CONNECTIONS: usize = 128;"));
        assert!(client.contains("const DEFAULT_MAX_KEEPALIVE_CONNECTIONS: usize = 16;"));
        assert!(client.contains("pub enum ClientError"));
        assert!(client.contains("impl fmt::Display for ClientError"));
        assert!(client.contains("impl std::error::Error for ClientError"));
        assert!(client.contains("fn acquire_slot(&self) -> ClientResult<SlotGuard>"));
        assert!(client.contains("http: reqwest::blocking::Client,"));
        assert!(client.contains("pub fn get_pet_by_id(&self, pet_id: i64) -> ClientResult<Pet> {"));
        assert!(client.contains("let url = format!(\"{}/pets/{}\", self.base_url, pet_id);"));
        assert!(client.contains("self.http.get(url)"));
        assert!(client.contains("let _slot = self.acquire_slot()?;"));
        assert!(client.contains("Ok(resp.json()?)"));
    }

    #[test]
    fn query_operation_uses_method_from_bytes_with_post_fallback_branch() {
        let out = generate(SPEC_32_QUERY, &opts()).unwrap();
        let client = file(&out, "client.rs");
        assert!(client.contains("use_post_fallback: bool,"));
        assert!(client.contains(
            "pub fn with_post_fallback(mut self, use_post_fallback: bool) -> Self {"
        ));
        assert!(client.contains("pub fn search_pets(&self, body:"));
        assert!(client.contains("let mut req = if self.use_post_fallback {"));
        assert!(client.contains("let twin_url = format!(\"{}/pets\", self.base_url);"));
        assert!(client.contains("self.http.post(twin_url)"));
        assert!(client.contains(
            "self.http.request(reqwest::Method::from_bytes(b\"QUERY\").expect(\"valid HTTP method\"), url)"
        ));
    }

    #[test]
    fn non_query_operation_keeps_direct_verb_dispatch() {
        let out = generate(SPEC, &opts()).unwrap();
        let client = file(&out, "client.rs");
        // `use_post_fallback` is always present on `Client` (shared runtime
        // surface), but a non-QUERY operation still dispatches directly.
        assert!(client.contains("use_post_fallback: bool,"));
        assert!(!client.contains("Method::from_bytes"));
        assert!(client.contains("self.http.get(url)"));
    }

    #[test]
    fn deterministic() {
        let a = generate(SPEC, &opts()).unwrap();
        let b = generate(SPEC, &opts()).unwrap();
        for (fa, fb) in a.files.iter().zip(b.files.iter()) {
            assert_eq!(fa.contents, fb.contents);
        }
    }
}
// CODEGEN-END
