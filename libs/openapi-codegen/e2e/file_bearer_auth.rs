// <HANDWRITE gap="missing-generator:file-bearer-auth-runtime-matrix" tracker="#3919" reason="executing generated Rust, Python, and TypeScript clients requires external toolchains and trap transports">

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use openapi_codegen::{
    generate_for_target_with_file_bearer_auth, FileBearerAuth, FileBearerScheme, GenOptions,
    GeneratedOutput, HttpClient, Lang, PythonTarget, RustTarget, TargetProfile, TypeScriptTarget,
};

const SPEC: &str = r##"{
  "openapi": "3.1.0",
  "info": { "title": "File bearer contract", "version": "1.0.0" },
  "paths": {
    "/ping": {
      "get": {
        "operationId": "ping",
        "responses": { "204": { "description": "ok" } }
      }
    }
  }
}"##;

fn opts(lang: Lang, target: TargetProfile) -> GenOptions {
    GenOptions {
        lang,
        target: Some(target),
        spec_path: PathBuf::new(),
        out_dir: PathBuf::new(),
        client_name: "createClient".to_string(),
        http_client: HttpClient::Fetch,
        emit_types: true,
        emit_client: true,
        emit_hooks: false,
    }
}

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "openapi-codegen-file-bearer-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("create file bearer temp directory");
    path
}

fn auth(token_path: &Path) -> FileBearerAuth {
    FileBearerAuth::new(
        token_path,
        ".svc.cluster.local",
        [FileBearerScheme::Http, FileBearerScheme::Https],
    )
    .expect("valid file bearer contract")
}

fn materialize(output: &GeneratedOutput, path: &Path) {
    output.write_to_dir(path).expect("write generated output");
    assert!(path.join(".openapi-codegen.json").is_file());
}

fn output(command: &mut Command, what: &str) -> Output {
    command
        .output()
        .unwrap_or_else(|error| panic!("{what}: failed to spawn: {error}"))
}

fn assert_success(output: Output, what: &str) {
    assert!(
        output.status.success(),
        "{what}: exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generated_python_rereads_and_fails_closed_before_sync_or_async_transport() {
    let root = temp_dir("python");
    let package = root.join("generated_api");
    let token_path = root.join("token");
    let target = TargetProfile::Python(PythonTarget::Py311);
    let generated = generate_for_target_with_file_bearer_auth(
        SPEC,
        &opts(Lang::Py, target),
        target,
        &auth(&token_path),
    )
    .expect("generate Python auth client");
    materialize(&generated, &package);

    let script = r#"
import asyncio
import os
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
token = pathlib.Path(sys.argv[2])
sys.path.insert(0, str(root))
from generated_api import AsyncClient, Client

class Response:
    def raise_for_status(self): pass
    def json(self): return None

class SyncTransport:
    def __init__(self): self.calls = []
    def request(self, method, url, **kwargs):
        self.calls.append((method, url, dict(kwargs["headers"])))
        return Response()

class AsyncTransport:
    def __init__(self): self.calls = []
    async def request(self, method, url, **kwargs):
        self.calls.append((method, url, dict(kwargs["headers"])))
        return Response()

def auth_value(headers):
    found = [value for name, value in headers.items() if name.casefold() == "authorization"]
    return found[0] if found else None

eligible = "http://lumen.apps.svc.cluster.local:7373"
token.write_text("one\n", encoding="utf-8")
sync = SyncTransport()
client = Client(eligible, client=sync)
client.ping()
token.write_text("two\n", encoding="utf-8")
client.ping()
assert [auth_value(call[2]) for call in sync.calls] == ["Bearer one", "Bearer two"]

token.unlink()
explicit = SyncTransport()
Client(eligible, client=explicit, default_headers={"aUtHoRiZaTiOn": "Bearer explicit"}).ping()
assert auth_value(explicit.calls[0][2]) == "Bearer explicit"

negative = [
    "http://svc.cluster.local:7373",
    "http://lumen.apps.svc:7373",
    "http://127.0.0.1:7373",
    "http://[::1]:7373",
    "http://host.docker.internal:7373",
    "http://lumen.apps.svc.cluster.local.evil:7373",
    "http://lumen.apps.svc.cluster.local.:7373",
    "http://lumen.apps.svc.cluster.local@evil.example:7373",
    "http://evil.example/lumen.apps.svc.cluster.local",
    "http://bad_.apps.svc.cluster.local:7373",
    "http://bad..apps.svc.cluster.local:7373",
    "http://lumen.apps.svc.cluster.local:7373?",
    "http://lumen.apps.svc.cluster.local:7373#",
    "ftp://lumen.apps.svc.cluster.local",
]
for base_url in negative:
    trap = SyncTransport()
    Client(base_url, client=trap).ping()
    assert auth_value(trap.calls[0][2]) is None, base_url

for contents in (None, "", "   \n"):
    if token.exists():
        if token.is_dir(): token.rmdir()
        else: token.unlink()
    if contents is not None: token.write_text(contents, encoding="utf-8")
    trap = SyncTransport()
    try:
        Client(eligible, client=trap).ping()
        raise AssertionError("missing token failure")
    except ValueError as error:
        assert "request authentication token" in str(error)
        assert not trap.calls

if token.exists():
    token.unlink()
token.mkdir()
trap = SyncTransport()
try:
    Client(eligible, client=trap).ping()
    raise AssertionError("directory token failure")
except ValueError:
    assert not trap.calls
token.rmdir()

canary = "secret-canary-must-not-leak"
token.write_text(canary + "\ninvalid", encoding="utf-8")
try:
    Client(eligible, client=SyncTransport()).ping()
    raise AssertionError("invalid token failure")
except ValueError as error:
    assert canary not in str(error)

async def async_contract():
    token.write_text("async-one", encoding="utf-8")
    transport = AsyncTransport()
    client = AsyncClient(eligible, client=transport)
    await client.ping()
    token.write_text("async-two", encoding="utf-8")
    await client.ping()
    assert [auth_value(call[2]) for call in transport.calls] == ["Bearer async-one", "Bearer async-two"]

asyncio.run(async_contract())
"#;
    let script_path = root.join("contract.py");
    fs::write(&script_path, script).expect("write Python runtime contract");
    let run = output(
        Command::new("uv")
            .arg("run")
            .arg("--quiet")
            .arg("--no-project")
            .arg("--python")
            .arg("3.13")
            .arg("--with")
            .arg("pydantic==2.12.5")
            .arg("python")
            .arg(&script_path)
            .arg(&root)
            .arg(&token_path)
            .env("UV_CACHE_DIR", "/private/tmp/lumen-0429-uv-cache")
            .env("PYTHONPYCACHEPREFIX", "/private/tmp/lumen-0429-pycache"),
        "execute generated Python file bearer client",
    );
    assert_success(run, "execute generated Python file bearer client");
    fs::remove_dir_all(root).expect("remove Python file bearer temp directory");
}

#[test]
fn generated_node_client_rereads_blocks_browser_and_never_calls_trap_fetch_on_error() {
    let root = temp_dir("typescript");
    let generated_dir = root.join("generated");
    let token_path = root.join("token");
    let target = TargetProfile::TypeScript(TypeScriptTarget::Ts50);
    let generated = generate_for_target_with_file_bearer_auth(
        SPEC,
        &opts(Lang::Ts, target),
        target,
        &auth(&token_path),
    )
    .expect("generate TypeScript auth client");
    materialize(&generated, &generated_dir);

    let script = r#"
import { mkdir, rm, writeFile } from "node:fs/promises";
import { request } from "./generated/runtime.ts";

const token = process.argv[2];
const eligible = "http://lumen.apps.svc.cluster.local:7373";
const calls: Array<Record<string, string>> = [];
const fakeFetch = async (_url: string, init?: any) => {
  calls.push({ ...(init?.headers ?? {}) });
  return { ok: true, status: 204, async json() { return undefined; } } as any;
};
const authValue = (headers: Record<string, string>) =>
  Object.entries(headers).find(([name]) => name.toLowerCase() === "authorization")?.[1];
const ping = (baseUrl: string, headers?: Record<string, string>, fetchImpl: any = fakeFetch) =>
  request<void>({ baseUrl, headers, fetch: fetchImpl }, { method: "GET", path: "/ping", expectBody: false });

await writeFile(token, "one\n", "utf8");
await ping(eligible);
await writeFile(token, "two\n", "utf8");
await ping(eligible);
if (authValue(calls[0]) !== "Bearer one" || authValue(calls[1]) !== "Bearer two") throw new Error("rotation");

await rm(token);
await ping(eligible, { aUtHoRiZaTiOn: "Bearer explicit" });
if (authValue(calls[2]) !== "Bearer explicit") throw new Error("explicit precedence");

const negative = [
  "http://svc.cluster.local:7373",
  "http://lumen.apps.svc:7373",
  "http://127.0.0.1:7373",
  "http://[::1]:7373",
  "http://host.docker.internal:7373",
  "http://lumen.apps.svc.cluster.local.evil:7373",
  "http://lumen.apps.svc.cluster.local.:7373",
  "http://lumen.apps.svc.cluster.local@evil.example:7373",
  "http://evil.example/lumen.apps.svc.cluster.local",
  "http://bad_.apps.svc.cluster.local:7373",
  "http://bad..apps.svc.cluster.local:7373",
  "http://lumen.apps.svc.cluster.local:7373?",
  "http://lumen.apps.svc.cluster.local:7373#",
  "ftp://lumen.apps.svc.cluster.local",
];
for (const baseUrl of negative) {
  const before = calls.length;
  await ping(baseUrl);
  if (authValue(calls[before])) throw new Error(`unexpected auth for ${baseUrl}`);
}

for (const contents of [null, "", "   \n"] as const) {
  await rm(token, { force: true, recursive: true });
  if (contents !== null) await writeFile(token, contents, "utf8");
  let trapCalls = 0;
  try {
    await ping(eligible, undefined, async () => { trapCalls += 1; throw new Error("transport called"); });
    throw new Error("missing token failure");
  } catch (error) {
    if (!String(error).includes("request authentication token")) throw error;
    if (trapCalls !== 0) throw new Error("transport ran before auth failure");
  }
}

await rm(token, { force: true, recursive: true });
await mkdir(token);
let directoryCalls = 0;
try {
  await ping(eligible, undefined, async () => { directoryCalls += 1; throw new Error("transport called"); });
  throw new Error("directory token failure");
} catch (error) {
  if (!String(error).includes("request authentication token") || directoryCalls !== 0) throw error;
}
await rm(token, { recursive: true });

const canary = "secret-canary-must-not-leak";
await writeFile(token, `${canary}\ninvalid`, "utf8");
try {
  await ping(eligible);
  throw new Error("invalid token failure");
} catch (error) {
  if (String(error).includes(canary)) throw new Error("token leaked in error");
}

await rm(token);
const savedProcess = (globalThis as any).process;
Object.defineProperty(globalThis, "process", { value: undefined, configurable: true, writable: true });
let browserCalls = 0;
try {
  await ping(eligible, undefined, async () => { browserCalls += 1; return {} as any; });
  throw new Error("eligible browser did not fail");
} catch (error) {
  if (!String(error).includes("unavailable in this runtime") || browserCalls !== 0) throw error;
}
await ping("http://localhost:7373", undefined, async (_url: string, init?: any) => {
  browserCalls += 1;
  if (authValue(init.headers)) throw new Error("browser external auth");
  return { ok: true, status: 204 } as any;
});
await ping(eligible, { Authorization: "Bearer browser-explicit" }, async (_url: string, init?: any) => {
  browserCalls += 1;
  if (authValue(init.headers) !== "Bearer browser-explicit") throw new Error("browser explicit");
  return { ok: true, status: 204 } as any;
});
Object.defineProperty(globalThis, "process", { value: savedProcess, configurable: true, writable: true });
"#;
    let script_path = root.join("contract.ts");
    fs::write(&script_path, script).expect("write TypeScript runtime contract");
    let run = output(
        Command::new("node")
            .arg("--experimental-strip-types")
            .arg("--no-warnings")
            .arg(&script_path)
            .arg(&token_path),
        "execute generated TypeScript file bearer client",
    );
    assert_success(run, "execute generated TypeScript file bearer client");
    fs::remove_dir_all(root).expect("remove TypeScript file bearer temp directory");
}

fn recording_proxy(
    expected: usize,
) -> (
    SocketAddr,
    mpsc::Receiver<Vec<Vec<String>>>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind recording proxy");
    let address = listener.local_addr().expect("recording proxy address");
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        for _ in 0..expected {
            let (mut stream, _) = listener.accept().expect("accept generated Rust request");
            stream
                .set_read_timeout(Some(Duration::from_secs(10)))
                .expect("set proxy read timeout");
            let mut bytes = Vec::new();
            let mut chunk = [0_u8; 1024];
            while !bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = stream
                    .read(&mut chunk)
                    .expect("read generated Rust request");
                if read == 0 {
                    break;
                }
                bytes.extend_from_slice(&chunk[..read]);
            }
            requests.push(
                String::from_utf8_lossy(&bytes)
                    .lines()
                    .map(str::to_string)
                    .collect(),
            );
            stream
                .write_all(
                    b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .expect("answer generated Rust request");
        }
        tx.send(requests).expect("send recorded Rust requests");
    });
    (address, rx, handle)
}

fn rust_authorization(lines: &[String]) -> Option<&str> {
    lines.iter().find_map(|line| {
        line.split_once(':').and_then(|(name, value)| {
            name.eq_ignore_ascii_case("authorization")
                .then_some(value.trim())
        })
    })
}

#[test]
fn generated_rust_client_rereads_and_honors_case_insensitive_explicit_authorization() {
    let root = temp_dir("rust");
    let source = root.join("src");
    let token_path = root.join("token");
    let target = TargetProfile::Rust(RustTarget::Rust2021);
    let generated = generate_for_target_with_file_bearer_auth(
        SPEC,
        &opts(Lang::Rust, target),
        target,
        &auth(&token_path),
    )
    .expect("generate Rust auth client");
    materialize(&generated, &source);
    fs::rename(source.join("mod.rs"), source.join("lib.rs")).expect("install Rust lib root");
    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "generated_file_bearer"
version = "0.0.0"
edition = "2021"

[dependencies]
reqwest = { version = "0.12", features = ["blocking", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
    )
    .expect("write generated Rust manifest");
    fs::write(
        source.join("main.rs"),
        r#"use generated_file_bearer::client::Client;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = &args[1];
    let base_url = &args[2];
    let token_path = &args[3];
    match mode.as_str() {
        "rotate" => {
            std::fs::write(token_path, "one\n").unwrap();
            Client::new(base_url).ping().unwrap();
            std::fs::write(token_path, "two\n").unwrap();
            Client::new(base_url).ping().unwrap();
        }
        "explicit" => {
            let header_name = &args[4];
            Client::new(base_url)
                .with_default_header(header_name, "Bearer explicit")
                .unwrap()
                .ping()
                .unwrap();
        }
        "call" => match Client::new(base_url).ping() {
            Ok(()) => println!("ok"),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(7);
            }
        },
        _ => panic!("unknown mode"),
    }
}
"#,
    )
    .expect("write generated Rust consumer");
    let build = output(
        Command::new("cargo")
            .arg("build")
            .arg("--quiet")
            .arg("--offline")
            .current_dir(&root)
            .env("CARGO_TARGET_DIR", root.join("target")),
        "compile generated Rust file bearer client",
    );
    assert_success(build, "compile generated Rust file bearer client");
    let binary = root.join("target/debug/generated_file_bearer");
    let eligible = "http://lumen.apps.svc.cluster.local";

    let (proxy, rx, handle) = recording_proxy(2);
    let proxy_url = format!("http://{proxy}");
    let rotation = output(
        Command::new(&binary)
            .args(["rotate", eligible, token_path.to_str().unwrap()])
            .env("HTTP_PROXY", &proxy_url)
            .env("http_proxy", &proxy_url)
            .env("NO_PROXY", "")
            .env("no_proxy", ""),
        "run generated Rust rotation client",
    );
    assert_success(rotation, "run generated Rust rotation client");
    let requests = rx
        .recv_timeout(Duration::from_secs(10))
        .expect("rotation requests");
    handle.join().expect("join rotation proxy");
    assert_eq!(rust_authorization(&requests[0]), Some("Bearer one"));
    assert_eq!(rust_authorization(&requests[1]), Some("Bearer two"));

    fs::remove_file(&token_path).expect("remove token before explicit test");
    let (proxy, rx, handle) = recording_proxy(1);
    let proxy_url = format!("http://{proxy}");
    let explicit = output(
        Command::new(&binary)
            .args([
                "explicit",
                eligible,
                token_path.to_str().unwrap(),
                "aUtHoRiZaTiOn",
            ])
            .env("HTTP_PROXY", &proxy_url)
            .env("http_proxy", &proxy_url)
            .env("NO_PROXY", "")
            .env("no_proxy", ""),
        "run generated Rust explicit-header client",
    );
    assert_success(explicit, "run generated Rust explicit-header client");
    let requests = rx
        .recv_timeout(Duration::from_secs(10))
        .expect("explicit request");
    handle.join().expect("join explicit proxy");
    assert_eq!(rust_authorization(&requests[0]), Some("Bearer explicit"));

    for contents in [None, Some(""), Some("   \n")] {
        let _ = fs::remove_dir_all(&token_path);
        let _ = fs::remove_file(&token_path);
        if let Some(contents) = contents {
            fs::write(&token_path, contents).expect("write invalid Rust token");
        }
        let failed = output(
            Command::new(&binary).args(["call", eligible, token_path.to_str().unwrap()]),
            "run generated Rust failing client",
        );
        assert_eq!(failed.status.code(), Some(7));
        let stderr = String::from_utf8_lossy(&failed.stderr);
        assert!(stderr.contains("request authentication token"));
        assert!(
            !stderr.contains(contents.unwrap_or("missing-token-canary")) || contents == Some("")
        );
    }

    let _ = fs::remove_file(&token_path);
    fs::create_dir(&token_path).expect("create directory token path");
    let directory = output(
        Command::new(&binary).args(["call", eligible, token_path.to_str().unwrap()]),
        "run generated Rust directory-token client",
    );
    assert_eq!(directory.status.code(), Some(7));
    fs::remove_dir(&token_path).expect("remove directory token path");

    let canary = "secret-canary-must-not-leak";
    fs::write(&token_path, format!("{canary}\ninvalid")).expect("write invalid token canary");
    let invalid = output(
        Command::new(&binary).args(["call", eligible, token_path.to_str().unwrap()]),
        "run generated Rust invalid-token client",
    );
    assert_eq!(invalid.status.code(), Some(7));
    assert!(!String::from_utf8_lossy(&invalid.stderr).contains(canary));

    fs::remove_file(&token_path).expect("remove token before negative host tests");
    for base_url in [
        "http://svc.cluster.local",
        "http://lumen.apps.svc.cluster.local.evil",
        "http://lumen.apps.svc.cluster.local.",
        "http://lumen.apps.svc.cluster.local@evil.example",
        "http://evil.example/lumen.apps.svc.cluster.local",
        "http://bad_.apps.svc.cluster.local",
        "http://bad..apps.svc.cluster.local",
        "http://lumen.apps.svc.cluster.local?",
        "http://lumen.apps.svc.cluster.local#",
        "http://127.0.0.1",
        "http://[::1]",
        "ftp://lumen.apps.svc.cluster.local",
    ] {
        let failed = output(
            Command::new(&binary).args(["call", base_url, token_path.to_str().unwrap()]),
            "run generated Rust negative host client",
        );
        let stderr = String::from_utf8_lossy(&failed.stderr);
        assert!(
            !stderr.contains("request authentication token"),
            "negative host read token: {base_url}: {stderr}"
        );
    }

    fs::remove_dir_all(root).expect("remove Rust file bearer temp directory");
}
