//! Python emitter: read an OpenAPI 3.0/3.1 document and emit pydantic v2 models
//! plus a typed sync/async HTTP/2 client runtime.
//!
//! Pipeline: parse → `models.py` (BaseModel per component schema) +
//! `h2c_runtime.py` (generated h2c + TLS ALPN h2 runtime) + `client.py`
//! (one sync and async `Client` method per operation) + `__init__.py`.

pub mod client_emit;
pub mod models_emit;
pub mod pymap;
pub mod runtime_emit;

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
            rel_path: "h2c_runtime.py".to_string(),
            contents: runtime_emit::emit(),
        });
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
        out.push_str("from .client import AsyncClient, Client  # noqa: F401\n");
        out.push_str("from .h2c_runtime import AsyncH2CClient, AsyncH2CConnection, AsyncH2CStream, H2CClient, H2CConnection, H2CResponse, H2CStream  # noqa: F401\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HttpClient, Lang};
    use std::fs;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::path::PathBuf;
    use std::process::Command;
    use std::thread;
    use std::time::{Duration, Instant};
    use std::time::{SystemTime, UNIX_EPOCH};

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

    const RECURSIVE_UNION_SPEC: &str = r##"{
      "openapi": "3.0.0",
      "info": { "title": "Mini", "version": "1.0.0" },
      "paths": {},
      "components": { "schemas": {
        "MatchQuery": {
          "type": "object",
          "properties": {
            "field": { "type": "string" },
            "text": { "type": "string" }
          },
          "required": ["field", "text"]
        },
        "TermQuery": {
          "type": "object",
          "properties": {
            "field": { "type": "string" },
            "value": { "type": "string" }
          },
          "required": ["field", "value"]
        },
        "QueryNode": {
          "oneOf": [
            {
              "type": "object",
              "required": ["match"],
              "properties": { "match": { "$ref": "#/components/schemas/MatchQuery" } }
            },
            {
              "type": "object",
              "required": ["term"],
              "properties": { "term": { "$ref": "#/components/schemas/TermQuery" } }
            },
            {
              "type": "object",
              "required": ["and"],
              "properties": {
                "and": {
                  "type": "array",
                  "items": { "$ref": "#/components/schemas/QueryNode" }
                }
              }
            },
            {
              "type": "object",
              "required": ["not"],
              "properties": { "not": { "$ref": "#/components/schemas/QueryNode" } }
            }
          ]
        },
        "SearchRequest": {
          "type": "object",
          "properties": { "query": { "$ref": "#/components/schemas/QueryNode" } },
          "required": ["query"]
        }
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
        assert_eq!(
            names,
            vec!["models.py", "h2c_runtime.py", "client.py", "__init__.py"]
        );
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
    fn pydantic_models_preserve_inline_oneof_object_variants() {
        let out = generate(RECURSIVE_UNION_SPEC, &opts()).unwrap();
        let models = file(&out, "models.py");
        assert!(models.contains("from pydantic import BaseModel, Field, RootModel"));
        assert!(models.contains("class QueryNodeMatch(BaseModel):"));
        assert!(models.contains("class QueryNodeTerm(BaseModel):"));
        assert!(models.contains("class QueryNodeAnd(BaseModel):"));
        assert!(models.contains("    and_: list[QueryNode] = Field(alias=\"and\")"));
        assert!(models.contains("class QueryNodeNot(BaseModel):"));
        assert!(models.contains("    not_: QueryNode = Field(alias=\"not\")"));
        assert!(models.contains(
            "class QueryNode(RootModel[QueryNodeMatch | QueryNodeTerm | QueryNodeAnd | QueryNodeNot]):"
        ));

        let dir = write_generated_python_package(&out);
        let script = format!(
            r#"
import sys
sys.path.insert(0, {dir:?})
from generated_api import QueryNode, QueryNodeAnd, QueryNodeMatch, SearchRequest

node = QueryNode.model_validate({{"and": [
    {{"match": {{"field": "body", "text": "hello"}}}},
    {{"not": {{"term": {{"field": "status", "value": "draft"}}}}}},
]}})
assert isinstance(node.root, QueryNodeAnd), type(node.root)
assert isinstance(node.root.and_[0].root, QueryNodeMatch), type(node.root.and_[0].root)
search = SearchRequest.model_validate({{"query": {{"match": {{"field": "body", "text": "hello"}}}}}})
assert isinstance(search.query.root, QueryNodeMatch), type(search.query.root)
QueryNode.model_json_schema()
"#,
            dir = dir.display().to_string(),
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python oneOf pydantic smoke");
        assert!(
            output.status.success(),
            "generated Python oneOf pydantic smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn h2c_default_client_method_is_typed_and_validated() {
        let out = generate(SPEC, &opts()).unwrap();
        let client = file(&out, "client.py");
        assert!(client.contains("class SupportsRequest(Protocol):"));
        assert!(client.contains("from .h2c_runtime import AsyncH2CClient, H2CClient"));
        assert!(client.contains("class Client:"));
        assert!(client.contains("client: Optional[SupportsRequest] = None"));
        assert!(client.contains("default_headers: Optional[Mapping[str, Any]] = None"));
        assert!(client.contains("auth_token: Optional[str] = None"));
        assert!(client.contains("self._client = client or H2CClient()"));
        assert!(
            client.contains("self._default_headers: dict[str, Any] = dict(default_headers or {})")
        );
        assert!(
            client.contains("self._default_headers[\"Authorization\"] = f\"Bearer {auth_token}\"")
        );
        assert!(client.contains("def __enter__(self) -> \"Client\":"));
        assert!(client.contains("def close(self) -> None:"));
        assert!(client.contains("def get_pet_by_id(self, *, pet_id: int) -> Pet:"));
        assert!(client.contains("_path = f\"/pets/{pet_id}\""));
        assert!(client.contains("_headers: dict[str, Any] = dict(self._default_headers)"));
        assert!(client.contains("self._client.request(\"GET\""));
        assert!(client.contains("return Pet.model_validate(_resp.json())"));
        assert!(client.contains("class AsyncSupportsRequest(Protocol):"));
        assert!(client.contains("class AsyncClient:"));
        assert!(client.contains("self._client = client or AsyncH2CClient()"));
        assert!(client.contains("async def __aenter__(self) -> \"AsyncClient\":"));
        assert!(client.contains("async def aclose(self) -> None:"));
        assert!(client.contains("async def get_pet_by_id(self, *, pet_id: int) -> Pet:"));
        assert!(client.contains("_resp = await self._client.request(\"GET\""));

        let init = file(&out, "__init__.py");
        assert!(init.contains("from .client import AsyncClient, Client"));
        assert!(init.contains("AsyncH2CClient, AsyncH2CConnection, AsyncH2CStream"));
        assert!(init.contains("H2CClient, H2CConnection, H2CResponse, H2CStream"));
    }

    #[test]
    fn generated_python_client_merges_auth_defaults_into_method_headers() {
        let out = generate(SPEC, &opts()).unwrap();
        let dir = write_generated_python_package(&out);
        let script = format!(
            r#"
import asyncio
import sys
sys.path.insert(0, {dir:?})
from generated_api.client import AsyncClient, Client

class Response:
    def raise_for_status(self):
        pass
    def json(self):
        return {{"id": 1, "name": "n"}}

class Fake:
    def __init__(self):
        self.calls = []
    def request(self, method, url, *, params, headers, json=None, data=None, content=None, timeout=None):
        self.calls.append((method, url, dict(headers)))
        return Response()

class AsyncFake:
    def __init__(self):
        self.calls = []
    async def request(self, method, url, *, params, headers, json=None, data=None, content=None, timeout=None):
        self.calls.append((method, url, dict(headers)))
        return Response()

sync = Fake()
Client("http://example", client=sync, default_headers={{"X-Trace": "t"}}, auth_token="tok").get_pet_by_id(pet_id=1)
assert sync.calls[0][2]["Authorization"] == "Bearer tok", sync.calls
assert sync.calls[0][2]["X-Trace"] == "t", sync.calls

async def main():
    transport = AsyncFake()
    await AsyncClient("http://example", client=transport, default_headers={{"Authorization": "Bearer explicit"}}).get_pet_by_id(pet_id=2)
    assert transport.calls[0][2]["Authorization"] == "Bearer explicit", transport.calls

asyncio.run(main())
"#,
            dir = dir.display().to_string(),
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python client auth-default smoke");
        assert!(
            output.status.success(),
            "generated Python client auth-default smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn runtime_exposes_unary_and_bidi_surfaces() {
        let out = generate(SPEC, &opts()).unwrap();
        let runtime = file(&out, "h2c_runtime.py");
        assert!(runtime.contains("import asyncio"));
        assert!(runtime.contains("import ssl"));
        assert!(runtime.contains("class H2CClient:"));
        assert!(runtime.contains("class H2CConnection:"));
        assert!(runtime.contains("TLS ALPN h2"));
        assert!(runtime.contains("selected_alpn_protocol()"));
        assert!(runtime.contains("def request("));
        assert!(runtime.contains("def get("));
        assert!(runtime.contains("def stream("));
        assert!(runtime.contains("max_connections_per_origin"));
        assert!(runtime.contains(
            "default_headers: Mapping[str, Any] | Iterable[tuple[str, Any]] | None = None"
        ));
        assert!(
            runtime.contains("self._default_headers[\"Authorization\"] = f\"Bearer {auth_token}\"")
        );
        assert!(runtime.contains("_DEFAULT_TIMEOUT = 5.0"));
        assert!(runtime.contains("_DEFAULT_MAX_RESPONSE_BYTES = 64 * 1024 * 1024"));
        assert!(runtime.contains("class H2CStream:"));
        assert!(runtime.contains("def send_data("));
        assert!(runtime.contains("def iter_json_lines("));
        assert!(runtime.contains("class AsyncH2CClient:"));
        assert!(runtime.contains("class AsyncH2CConnection:"));
        assert!(runtime.contains("class AsyncH2CStream:"));
        assert!(runtime.contains("class _AsyncH2CStreamContext:"));
        assert!(runtime.contains("async def request("));
        assert!(runtime.contains("async def iter_json_lines("));
        assert!(runtime.contains("def _decode_huffman("));
        assert!(runtime.contains("import socket"));
        assert!(!runtime.contains("from h2"));
        assert!(!runtime.contains("pip install h2"));
    }

    #[test]
    fn generated_python_hpack_encoder_uses_static_table_indexes() {
        let out = generate(SPEC, &opts()).unwrap();
        let dir = write_generated_python_package(&out);
        let script = format!(
            r#"
import sys
sys.path.insert(0, {dir:?} + "/generated_api")
from h2c_runtime import _encode_headers
encoded = _encode_headers([
    (":method", "GET"),
    (":scheme", "http"),
    (":path", "/"),
    ("accept", "application/json"),
    ("content-type", "application/json"),
])
assert encoded[:3] == b"\x82\x86\x84", encoded
assert b":method" not in encoded
assert b":scheme" not in encoded
assert b":path" not in encoded
assert b"accept" not in encoded
assert b"content-type" not in encoded
assert len(encoded) < 50, encoded
"#,
            dir = dir.display().to_string(),
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python HPACK static table smoke");
        assert!(
            output.status.success(),
            "generated Python HPACK static table smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generated_python_query_params_encode_bool_wire_values() {
        let out = generate(SPEC, &opts()).unwrap();
        let dir = write_generated_python_package(&out);
        let script = format!(
            r#"
import sys
sys.path.insert(0, {dir:?} + "/generated_api")
from h2c_runtime import _url_with_params

url = _url_with_params("http://127.0.0.1/items?existing=1", {{"force": True, "dry": False, "skip": None}})
assert url == "http://127.0.0.1/items?existing=1&force=true&dry=false", url

url = _url_with_params("http://127.0.0.1/items", [("flag", [True, False])])
assert url == "http://127.0.0.1/items?flag=true&flag=false", url
"#,
            dir = dir.display().to_string(),
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python query param encoding smoke");
        assert!(
            output.status.success(),
            "generated Python query param encoding smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generated_python_h2_runtime_rejects_unsafe_protocol_inputs() {
        let out = generate(SPEC, &opts()).unwrap();
        let dir = write_generated_python_package(&out);
        let script = format!(
            r#"
import sys
sys.path.insert(0, {dir:?} + "/generated_api")
from h2c_runtime import (
    H2CConnection,
    H2CProtocolError,
    H2CStream,
    _FLAG_END_HEADERS,
    _HpackDecoder,
    _MAX_FLOW_CONTROL_WINDOW,
    _MAX_HEADER_BLOCK,
    _MAX_INBOUND_FRAME,
    _encode_int,
    _header_pairs,
    _validate_method,
)

def must_reject(callback):
    try:
        callback()
    except H2CProtocolError:
        return
    except Exception as exc:
        raise AssertionError(f"expected H2CProtocolError, got {{type(exc).__name__}}: {{exc}}") from exc
    raise AssertionError("expected H2CProtocolError")

must_reject(lambda: _header_pairs({{"bad name": "x"}}))
must_reject(lambda: _header_pairs({{"x-test": "ok\r\nbad: yes"}}))
must_reject(lambda: _validate_method("GET /bad"))
must_reject(lambda: _HpackDecoder().decode(_encode_int(4097, 5, 0x20)))

conn = H2CConnection("http", "127.0.0.1", 1)
must_reject(lambda: conn._headers_payload(_FLAG_END_HEADERS, 1, b"x" * (_MAX_HEADER_BLOCK + 1)))
must_reject(lambda: conn._handle_settings(0, b"\x00\x04" + (_MAX_FLOW_CONTROL_WINDOW + 1).to_bytes(4, "big")))
must_reject(lambda: conn._handle_settings(0, b"\x00\x05\x00\x00\x00\x01"))

conn = H2CConnection("http", "127.0.0.1", 1)
conn._conn_send_window = _MAX_FLOW_CONTROL_WINDOW
must_reject(lambda: conn._handle_window_update(0, (1).to_bytes(4, "big")))

limited = H2CConnection("http", "127.0.0.1", 1, max_response_bytes=1)
stream = H2CStream(limited, 1)
stream.status_code = 200
stream._chunks.append(b"ok")
stream._response_ended = True
must_reject(stream.read_response)

stream = H2CStream(H2CConnection("http", "127.0.0.1", 1), 1)
must_reject(lambda: stream._handle_headers([("bad name", "x")]))
must_reject(lambda: stream._handle_headers([("x-test", "ok\nbad")]))

class BigFrameSock:
    def __init__(self):
        self.buf = (_MAX_INBOUND_FRAME + 1).to_bytes(3, "big") + b"\x00\x00\x00\x00\x00\x01"

    def recv(self, n):
        out = self.buf[:n]
        self.buf = self.buf[n:]
        return out

conn = H2CConnection("http", "127.0.0.1", 1)
conn._sock = BigFrameSock()
must_reject(conn._read_frame)
"#,
            dir = dir.display().to_string(),
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python h2 security smoke");
        assert!(
            output.status.success(),
            "generated Python h2 security smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generated_python_files_compile_when_python3_available() {
        let out = generate(SPEC, &opts()).unwrap();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("openapi-codegen-py-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        for generated in &out.files {
            fs::write(dir.join(&generated.rel_path), &generated.contents).unwrap();
        }

        let status = match Command::new("python3")
            .arg("-m")
            .arg("py_compile")
            .arg(dir.join("models.py"))
            .arg(dir.join("h2c_runtime.py"))
            .arg(dir.join("client.py"))
            .arg(dir.join("__init__.py"))
            .status()
        {
            Ok(status) => status,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return,
            Err(err) => panic!("failed to run python3: {err}"),
        };
        assert!(status.success(), "generated Python files failed py_compile");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generated_python_h2c_client_smoke_talks_to_real_h2c_frames() {
        let Ok(status) = Command::new("python3")
            .arg("-c")
            .arg("import pydantic")
            .status()
        else {
            return;
        };
        if !status.success() {
            return;
        }

        let out = generate(SPEC, &opts()).unwrap();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("openapi-codegen-py-{}-{nonce}", std::process::id()));
        let pkg = dir.join("generated_api");
        fs::create_dir_all(&pkg).unwrap();
        for generated in &out.files {
            fs::write(pkg.join(&generated.rel_path), &generated.contents).unwrap();
        }

        let (base_url, server) = spawn_h2c_smoke_server();
        let script = format!(
            r#"
import sys
sys.path.insert(0, {dir:?})
from generated_api import Client
pet = Client({base_url:?}).get_pet_by_id(pet_id=42)
assert pet.id == 42
assert pet.name == "Ada"
assert pet.tag == "h2c"
"#,
            dir = dir.display().to_string(),
            base_url = base_url,
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python h2c smoke");
        let server_result = server.join();
        assert!(
            output.status.success(),
            "generated Python h2c smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        server_result.expect("h2c smoke server panicked");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generated_python_async_h2c_runtime_smoke_talks_to_real_h2c_frames() {
        let out = generate(SPEC, &opts()).unwrap();
        let dir = write_generated_python_package(&out);
        let (base_url, server) = spawn_h2c_smoke_server();
        let script = format!(
            r#"
import asyncio
import sys
sys.path.insert(0, {dir:?} + "/generated_api")
from h2c_runtime import AsyncH2CClient
async def main():
    async with AsyncH2CClient(timeout=5) as client:
        response = await client.get({base_url:?} + "/pets/42")
        payload = response.json()
        assert payload["id"] == 42
        assert payload["name"] == "Ada"
        assert payload["tag"] == "h2c"
asyncio.run(main())
"#,
            dir = dir.display().to_string(),
            base_url = base_url,
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python async h2c smoke");
        let server_result = server.join();
        assert!(
            output.status.success(),
            "generated Python async h2c smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        server_result.expect("h2c async smoke server panicked");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generated_python_async_client_smoke_validates_pydantic_response() {
        let Ok(status) = Command::new("python3")
            .arg("-c")
            .arg("import pydantic")
            .status()
        else {
            return;
        };
        if !status.success() {
            return;
        }

        let out = generate(SPEC, &opts()).unwrap();
        let dir = write_generated_python_package(&out);
        let (base_url, server) = spawn_h2c_smoke_server();
        let script = format!(
            r#"
import asyncio
import sys
sys.path.insert(0, {dir:?})
from generated_api import AsyncClient
async def main():
    async with AsyncClient({base_url:?}) as client:
        pet = await client.get_pet_by_id(pet_id=42)
        assert pet.id == 42
        assert pet.name == "Ada"
        assert pet.tag == "h2c"
asyncio.run(main())
"#,
            dir = dir.display().to_string(),
            base_url = base_url,
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python async client smoke");
        let server_result = server.join();
        assert!(
            output.status.success(),
            "generated Python async client smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        server_result.expect("h2c async client smoke server panicked");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generated_python_async_h2c_runtime_supports_bidi_streaming() {
        let out = generate(SPEC, &opts()).unwrap();
        let dir = write_generated_python_package(&out);
        let (base_url, server) = spawn_h2c_bidi_server();
        let script = format!(
            r#"
import asyncio
import sys
sys.path.insert(0, {dir:?} + "/generated_api")
from h2c_runtime import AsyncH2CClient
async def main():
    async with AsyncH2CClient(timeout=5) as client:
        async with client.stream("POST", {base_url:?} + "/bidi") as stream:
            await stream.send_data("one\n")
            assert await stream.read_chunk() == b"ack:one\n"
            await stream.send_data("two\n", end_stream=True)
            chunks = []
            async for chunk in stream.iter_bytes():
                chunks.append(chunk)
    assert b"".join(chunks) == b"ack:two\n"
asyncio.run(main())
"#,
            dir = dir.display().to_string(),
            base_url = base_url,
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python async h2c bidi smoke");
        let server_result = server.join();
        assert!(
            output.status.success(),
            "generated Python async h2c bidi smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        server_result.expect("h2c async bidi server panicked");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generated_python_h2_runtime_negotiates_tls_alpn_h2() {
        let out = generate(SPEC, &opts()).unwrap();
        let dir = write_generated_python_package(&out);
        let cert = dir.join("cert.pem");
        let key = dir.join("key.pem");
        let Ok(output) = Command::new("openssl")
            .args(["req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout"])
            .arg(&key)
            .args(["-out"])
            .arg(&cert)
            .args(["-subj", "/CN=localhost", "-days", "1"])
            .output()
        else {
            let _ = fs::remove_dir_all(&dir);
            return;
        };
        if !output.status.success() {
            let _ = fs::remove_dir_all(&dir);
            return;
        }

        let script = format!(
            r#"
import asyncio
import socket
import ssl
import sys
import threading
import time
sys.path.insert(0, {dir:?} + "/generated_api")
from h2c_runtime import AsyncH2CClient, H2CClient

def write_frame(conn, kind, flags, stream_id, payload):
    conn.sendall(len(payload).to_bytes(3, "big") + bytes([kind, flags]) + (stream_id & 0x7fffffff).to_bytes(4, "big") + payload)

def read_exact(conn, n):
    out = bytearray()
    while len(out) < n:
        chunk = conn.recv(n - len(out))
        if not chunk:
            raise RuntimeError("connection closed")
        out.extend(chunk)
    return bytes(out)

def read_frame(conn):
    head = read_exact(conn, 9)
    size = int.from_bytes(head[:3], "big")
    return head[3], head[4], int.from_bytes(head[5:9], "big") & 0x7fffffff, read_exact(conn, size)

def enc_int(value, prefix_bits, prefix=0):
    max_prefix = (1 << prefix_bits) - 1
    if value < max_prefix:
        return bytes([prefix | value])
    out = bytearray([prefix | max_prefix])
    value -= max_prefix
    while value >= 128:
        out.append((value % 128) | 0x80)
        value //= 128
    out.append(value)
    return bytes(out)

def enc_str(value):
    raw = value.encode("utf-8")
    return enc_int(len(raw), 7, 0) + raw

def literal(name, value):
    return b"\x00" + enc_str(name) + enc_str(value)

def response_headers(length):
    return b"\x88" + literal("content-type", "application/json") + literal("content-length", str(length))

def serve_once(label):
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    port = listener.getsockname()[1]
    def run():
        raw, _ = listener.accept()
        ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        ctx.load_cert_chain({cert:?}, {key:?})
        ctx.set_alpn_protocols(["h2"])
        conn = ctx.wrap_socket(raw, server_side=True)
        assert conn.selected_alpn_protocol() == "h2"
        assert read_exact(conn, 24) == b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"
        write_frame(conn, 4, 0, 0, b"")
        while True:
            kind, flags, stream_id, payload = read_frame(conn)
            if kind == 4 and not (flags & 0x1):
                write_frame(conn, 4, 0x1, 0, b"")
            elif kind == 1:
                body = ("{{\"alpn\":\"h2\",\"client\":\"%s\"}}" % label).encode("utf-8")
                write_frame(conn, 1, 0x4, stream_id, response_headers(len(body)))
                write_frame(conn, 0, 0x1, stream_id, body)
                time.sleep(0.1)
                conn.close()
                listener.close()
                return
    thread = threading.Thread(target=run)
    thread.start()
    return port, thread

client_ctx = ssl.create_default_context()
client_ctx.check_hostname = False
client_ctx.verify_mode = ssl.CERT_NONE
port, thread = serve_once("sync")
client = H2CClient(timeout=5, ssl_context=client_ctx)
sync_payload = client.get(f"https://127.0.0.1:{{port}}/alpn").json()
client.close()
thread.join()
assert sync_payload == {{"alpn": "h2", "client": "sync"}}

async def main():
    async_ctx = ssl.create_default_context()
    async_ctx.check_hostname = False
    async_ctx.verify_mode = ssl.CERT_NONE
    port, thread = serve_once("async")
    async with AsyncH2CClient(timeout=5, ssl_context=async_ctx) as client:
        response = await client.get(f"https://127.0.0.1:{{port}}/alpn")
        payload = response.json()
    thread.join()
    assert payload == {{"alpn": "h2", "client": "async"}}

asyncio.run(main())
"#,
            dir = dir.display().to_string(),
            cert = cert.display().to_string(),
            key = key.display().to_string(),
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python TLS ALPN h2 smoke");
        assert!(
            output.status.success(),
            "generated Python TLS ALPN h2 smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generated_python_h2c_runtime_reuses_one_connection_for_sequential_requests() {
        let out = generate(SPEC, &opts()).unwrap();
        let dir = write_generated_python_package(&out);
        let (base_url, server) = spawn_h2c_sequential_server(2);
        let script = format!(
            r#"
import sys
sys.path.insert(0, {dir:?} + "/generated_api")
from h2c_runtime import H2CClient
client = H2CClient(timeout=5)
first = client.get({base_url:?} + "/first").json()
second = client.get({base_url:?} + "/second").json()
client.close()
assert first["stream"] == 1
assert second["stream"] == 3
"#,
            dir = dir.display().to_string(),
            base_url = base_url,
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python sequential h2c reuse smoke");
        let server_result = server.join();
        assert!(
            output.status.success(),
            "generated Python sequential h2c reuse smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        server_result.expect("h2c sequential server panicked");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generated_python_h2c_runtime_reused_connection_perf_guard() {
        let out = generate(SPEC, &opts()).unwrap();
        let dir = write_generated_python_package(&out);
        let request_count = 128_usize;
        let (base_url, server) = spawn_h2c_sequential_server(request_count);
        let script = format!(
            r#"
import sys
import time
sys.path.insert(0, {dir:?} + "/generated_api")
from h2c_runtime import H2CClient
client = H2CClient(timeout=5)
started = time.perf_counter()
for index in range({request_count}):
    payload = client.get({base_url:?} + f"/perf/{{index}}").json()
    assert payload["stream"] == index * 2 + 1
elapsed = time.perf_counter() - started
client.close()
print(f"h2c_generated_sync_reused_connection seconds={{elapsed:.6f}} requests={request_count}")
assert elapsed < 5.0, elapsed
"#,
            dir = dir.display().to_string(),
            base_url = base_url,
            request_count = request_count,
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python h2c perf guard");
        let server_result = server.join();
        assert!(
            output.status.success(),
            "generated Python h2c perf guard failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        eprintln!("{}", String::from_utf8_lossy(&output.stdout).trim());
        server_result.expect("h2c perf guard server panicked");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generated_python_h2c_runtime_multiplexes_concurrent_streams() {
        let out = generate(SPEC, &opts()).unwrap();
        let dir = write_generated_python_package(&out);
        let (base_url, server) = spawn_h2c_multiplex_server();
        let script = format!(
            r#"
import sys
from concurrent.futures import ThreadPoolExecutor
sys.path.insert(0, {dir:?} + "/generated_api")
from h2c_runtime import H2CClient
client = H2CClient(timeout=5)
def fetch(path):
    return client.get({base_url:?} + path).json()["stream"]
with ThreadPoolExecutor(max_workers=2) as pool:
    results = list(pool.map(fetch, ["/slow", "/fast"]))
client.close()
assert sorted(results) == [1, 3], results
"#,
            dir = dir.display().to_string(),
            base_url = base_url,
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python h2c multiplex smoke");
        let server_result = server.join();
        assert!(
            output.status.success(),
            "generated Python h2c multiplex smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        server_result.expect("h2c multiplex server panicked");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn generated_python_h2c_runtime_supports_bidi_streaming() {
        let out = generate(SPEC, &opts()).unwrap();
        let dir = write_generated_python_package(&out);
        let (base_url, server) = spawn_h2c_bidi_server();
        let script = format!(
            r#"
import sys
sys.path.insert(0, {dir:?} + "/generated_api")
from h2c_runtime import H2CClient
client = H2CClient(timeout=5)
with client.stream("POST", {base_url:?} + "/bidi") as stream:
    stream.send_data("one\n")
    assert stream.read_chunk() == b"ack:one\n"
    stream.send_data("two\n", end_stream=True)
    rest = b"".join(stream.iter_bytes())
client.close()
assert rest == b"ack:two\n"
"#,
            dir = dir.display().to_string(),
            base_url = base_url,
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run generated Python h2c bidi smoke");
        let server_result = server.join();
        assert!(
            output.status.success(),
            "generated Python h2c bidi smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        server_result.expect("h2c bidi server panicked");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn deterministic() {
        let a = generate(SPEC, &opts()).unwrap();
        let b = generate(SPEC, &opts()).unwrap();
        for (fa, fb) in a.files.iter().zip(b.files.iter()) {
            assert_eq!(fa.contents, fb.contents);
        }
    }

    fn write_generated_python_package(out: &GeneratedOutput) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("openapi-codegen-py-{}-{nonce}", std::process::id()));
        let pkg = dir.join("generated_api");
        fs::create_dir_all(&pkg).unwrap();
        for generated in &out.files {
            fs::write(pkg.join(&generated.rel_path), &generated.contents).unwrap();
        }
        dir
    }

    fn spawn_h2c_smoke_server() -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let started = Instant::now();
            let (mut stream, _) = loop {
                match listener.accept() {
                    Ok(accepted) => break accepted,
                    Err(err)
                        if err.kind() == std::io::ErrorKind::WouldBlock
                            && started.elapsed() < Duration::from_secs(5) =>
                    {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => panic!("h2c smoke server accept failed: {err}"),
                }
            };
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut preface = [0_u8; 24];
            stream.read_exact(&mut preface).unwrap();
            assert_eq!(&preface, b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");
            write_frame(&mut stream, 4, 0, 0, &[]);

            loop {
                let (kind, flags, stream_id, payload) = read_frame(&mut stream);
                match kind {
                    4 if flags & 0x1 == 0 => write_frame(&mut stream, 4, 0x1, 0, &[]),
                    1 => {
                        assert_eq!(stream_id, 1);
                        assert!(
                            !payload.is_empty(),
                            "client HEADERS payload should carry HPACK block"
                        );
                        let body = br#"{"id":42,"name":"Ada","tag":"h2c"}"#;
                        let headers = response_headers(body.len());
                        write_frame(&mut stream, 1, 0x4, stream_id, &headers);
                        write_frame(&mut stream, 0, 0x1, stream_id, body);
                        thread::sleep(Duration::from_millis(200));
                        return;
                    }
                    _ => {}
                }
            }
        });
        (format!("http://{addr}"), handle)
    }

    fn spawn_h2c_sequential_server(expected_requests: usize) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let mut stream = accept_h2c(listener, "sequential");
            let mut served = 0_usize;
            while served < expected_requests {
                let (kind, flags, stream_id, payload) = read_frame(&mut stream);
                match kind {
                    4 if flags & 0x1 == 0 => write_frame(&mut stream, 4, 0x1, 0, &[]),
                    1 => {
                        assert!(
                            !payload.is_empty(),
                            "client HEADERS payload should carry HPACK block"
                        );
                        let body = format!(r#"{{"stream":{stream_id}}}"#);
                        let headers = response_headers(body.len());
                        write_frame(&mut stream, 1, 0x4, stream_id, &headers);
                        write_frame(&mut stream, 0, 0x1, stream_id, body.as_bytes());
                        served += 1;
                    }
                    _ => {}
                }
            }
            thread::sleep(Duration::from_millis(100));
        });
        (format!("http://{addr}"), handle)
    }

    fn spawn_h2c_multiplex_server() -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let mut stream = accept_h2c(listener, "multiplex");
            let mut stream_ids = Vec::new();
            while stream_ids.len() < 2 {
                let (kind, flags, stream_id, payload) = read_frame(&mut stream);
                match kind {
                    4 if flags & 0x1 == 0 => write_frame(&mut stream, 4, 0x1, 0, &[]),
                    1 => {
                        assert!(
                            !payload.is_empty(),
                            "client HEADERS payload should carry HPACK block"
                        );
                        stream_ids.push(stream_id);
                    }
                    _ => {}
                }
            }
            for stream_id in stream_ids.iter().rev() {
                let body = format!(r#"{{"stream":{stream_id}}}"#);
                let headers = response_headers(body.len());
                write_frame(&mut stream, 1, 0x4, *stream_id, &headers);
                write_frame(&mut stream, 0, 0x1, *stream_id, body.as_bytes());
            }
            thread::sleep(Duration::from_millis(100));
        });
        (format!("http://{addr}"), handle)
    }

    fn spawn_h2c_bidi_server() -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let mut stream = accept_h2c(listener, "bidi");
            let mut response_started = false;
            loop {
                let (kind, flags, stream_id, payload) = read_frame(&mut stream);
                match kind {
                    4 if flags & 0x1 == 0 => write_frame(&mut stream, 4, 0x1, 0, &[]),
                    1 => {
                        assert_eq!(stream_id, 1);
                        assert!(
                            !payload.is_empty(),
                            "client HEADERS payload should carry HPACK block"
                        );
                        let headers = streaming_response_headers();
                        write_frame(&mut stream, 1, 0x4, stream_id, &headers);
                        response_started = true;
                    }
                    0 if stream_id == 1 => {
                        assert!(response_started, "response HEADERS should be sent first");
                        if !payload.is_empty() {
                            let body = format!("ack:{}", String::from_utf8_lossy(&payload));
                            write_frame(
                                &mut stream,
                                0,
                                if flags & 0x1 != 0 { 0x1 } else { 0 },
                                stream_id,
                                body.as_bytes(),
                            );
                        } else if flags & 0x1 != 0 {
                            write_frame(&mut stream, 0, 0x1, stream_id, &[]);
                        }
                        if flags & 0x1 != 0 {
                            thread::sleep(Duration::from_millis(100));
                            return;
                        }
                    }
                    _ => {}
                }
            }
        });
        (format!("http://{addr}"), handle)
    }

    fn accept_h2c(listener: TcpListener, label: &str) -> TcpStream {
        let started = Instant::now();
        let (mut stream, _) = loop {
            match listener.accept() {
                Ok(accepted) => break accepted,
                Err(err)
                    if err.kind() == std::io::ErrorKind::WouldBlock
                        && started.elapsed() < Duration::from_secs(5) =>
                {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("h2c {label} server accept failed: {err}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut preface = [0_u8; 24];
        stream.read_exact(&mut preface).unwrap();
        assert_eq!(&preface, b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");
        write_frame(&mut stream, 4, 0, 0, &[]);
        stream
    }

    fn read_frame(stream: &mut TcpStream) -> (u8, u8, u32, Vec<u8>) {
        let mut head = [0_u8; 9];
        stream.read_exact(&mut head).unwrap();
        let len = ((head[0] as usize) << 16) | ((head[1] as usize) << 8) | head[2] as usize;
        let kind = head[3];
        let flags = head[4];
        let stream_id = u32::from_be_bytes([head[5], head[6], head[7], head[8]]) & 0x7fff_ffff;
        let mut payload = vec![0_u8; len];
        stream.read_exact(&mut payload).unwrap();
        (kind, flags, stream_id, payload)
    }

    fn write_frame(stream: &mut TcpStream, kind: u8, flags: u8, stream_id: u32, payload: &[u8]) {
        assert!(payload.len() <= 0x00ff_ffff);
        let len = payload.len();
        let mut head = [0_u8; 9];
        head[0] = ((len >> 16) & 0xff) as u8;
        head[1] = ((len >> 8) & 0xff) as u8;
        head[2] = (len & 0xff) as u8;
        head[3] = kind;
        head[4] = flags;
        head[5..9].copy_from_slice(&(stream_id & 0x7fff_ffff).to_be_bytes());
        stream.write_all(&head).unwrap();
        stream.write_all(payload).unwrap();
    }

    fn response_headers(content_len: usize) -> Vec<u8> {
        let mut out = vec![0x88]; // indexed static :status 200
        hpack_literal(&mut out, "content-type", "application/json");
        hpack_literal(&mut out, "content-length", &content_len.to_string());
        out
    }

    fn streaming_response_headers() -> Vec<u8> {
        let mut out = vec![0x88]; // indexed static :status 200
        hpack_literal(&mut out, "content-type", "application/json");
        out
    }

    fn hpack_literal(out: &mut Vec<u8>, name: &str, value: &str) {
        out.push(0x00);
        hpack_huffman_string(out, name);
        hpack_huffman_string(out, value);
    }

    fn hpack_huffman_string(out: &mut Vec<u8>, value: &str) {
        let bytes: &[u8] = match value {
            "content-type" => &[0x21, 0xea, 0x49, 0x6a, 0x4a, 0xc9, 0xf5, 0x59, 0x7f],
            "application/json" => &[
                0x1d, 0x75, 0xd0, 0x62, 0x0d, 0x26, 0x3d, 0x4c, 0x74, 0x41, 0xea,
            ],
            "content-length" => &[0x21, 0xea, 0x49, 0x6a, 0x4a, 0xd4, 0x16, 0xa9, 0x93, 0x3f],
            "34" => &[0x65, 0xaf],
            _ => {
                hpack_string(out, value);
                return;
            }
        };
        hpack_int(out, bytes.len(), 7, 0x80);
        out.extend_from_slice(bytes);
    }

    fn hpack_string(out: &mut Vec<u8>, value: &str) {
        let bytes = value.as_bytes();
        hpack_int(out, bytes.len(), 7, 0);
        out.extend_from_slice(bytes);
    }

    fn hpack_int(out: &mut Vec<u8>, mut value: usize, prefix_bits: u8, prefix: u8) {
        let max_prefix = (1_usize << prefix_bits) - 1;
        if value < max_prefix {
            out.push(prefix | value as u8);
            return;
        }
        out.push(prefix | max_prefix as u8);
        value -= max_prefix;
        while value >= 128 {
            out.push(((value % 128) as u8) | 0x80);
            value /= 128;
        }
        out.push(value as u8);
    }
}
