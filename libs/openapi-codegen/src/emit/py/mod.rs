//! Python emitter: read an OpenAPI 3.0/3.1 document and emit pydantic v2 models
//! plus a typed `httpx` client.
//!
//! Pipeline: parse → `models.py` (BaseModel per component schema) + `client.py`
//! (one `Client` method per operation) + `__init__.py`.

pub mod client_emit;
pub mod models_emit;
pub mod pymap;

use crate::ir::build_type_map;
use crate::ir::openapi::Spec;
use crate::ir::operations;
use crate::{GenOptions, GeneratedFile, GeneratedOutput};
use anyhow::{Context, Result};

/// Pure Python generation: spec JSON text → in-memory files. No filesystem access.
pub fn generate(spec_json: &str, opts: &GenOptions) -> Result<GeneratedOutput> {
    let spec: Spec = serde_json::from_str(spec_json).context("failed to parse OpenAPI spec")?;
    let tm = build_type_map(&spec);
    let ops = operations::build(&spec);

    let mut files = Vec::new();
    if opts.emit_types {
        files.push(GeneratedFile {
            rel_path: "models.py".to_string(),
            contents: models_emit::emit(&spec, &tm),
        });
    }
    if opts.emit_client {
        files.push(GeneratedFile {
            rel_path: "client.py".to_string(),
            contents: client_emit::emit(&ops, &tm),
        });
    }
    files.push(GeneratedFile {
        rel_path: "__init__.py".to_string(),
        contents: emit_init(opts),
    });
    Ok(GeneratedOutput { files })
}

fn emit_init(opts: &GenOptions) -> String {
    let mut out = String::from(models_emit::HEADER);
    if opts.emit_types {
        out.push_str("from .models import *  # noqa: F401,F403\n");
    }
    if opts.emit_client {
        out.push_str("from .client import Client  # noqa: F401\n");
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
        "Pet": { "type": "object", "properties": { "id": { "type": "integer" }, "name": { "type": "string" }, "tag": { "type": "string" } }, "required": ["id", "name"] }
      } }
    }"##;

    fn opts() -> GenOptions {
        GenOptions {
            lang: Lang::Py,
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
    fn emits_models_client_init() {
        let out = generate(SPEC, &opts()).unwrap();
        let names: Vec<&str> = out.files.iter().map(|f| f.rel_path.as_str()).collect();
        assert_eq!(names, vec!["models.py", "client.py", "__init__.py"]);
    }

    #[test]
    fn pydantic_model_has_typed_required_and_optional_fields() {
        let out = generate(SPEC, &opts()).unwrap();
        let models = file(&out, "models.py");
        assert!(models.contains("from pydantic import BaseModel, Field"));
        assert!(models.contains("class Pet(BaseModel):"));
        assert!(models.contains("    id: int\n"));
        assert!(models.contains("    name: str\n"));
        assert!(models.contains("    tag: Optional[str] = None\n"));
    }

    #[test]
    fn httpx_client_method_is_typed_and_validated() {
        let out = generate(SPEC, &opts()).unwrap();
        let client = file(&out, "client.py");
        assert!(client.contains("import httpx"));
        assert!(client.contains("class Client:"));
        assert!(client.contains("def get_pet_by_id(self, *, pet_id: int) -> Pet:"));
        assert!(client.contains("_path = f\"/pets/{pet_id}\""));
        assert!(client.contains("self._client.request(\"GET\""));
        assert!(client.contains("return Pet.model_validate(_resp.json())"));
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
