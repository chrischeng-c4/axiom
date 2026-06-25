// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-src.md#schema
// CODEGEN-BEGIN
//! Built-in HTTP mock + record/replay proxy — the mock-killer.
//!
//! A transparent forward proxy: the runner gets `HTTP(S)_PROXY` and a CA-trust
//! bundle (see [`ca`]), so its outbound calls — plain HTTP and, via CONNECT +
//! MITM, HTTPS — are intercepted with **zero app code change**. Each request
//! resolves: a registered [`stub`] wins; else a registered OpenAPI spec (see
//! [`crate::emulator::openapi`]) answers from the contract; else a recorded
//! [`cassette`] replays; else (auto/record) it forwards to the real upstream and
//! records. `/__admin/*` origin-form requests are the control API (stubs +
//! `/__admin/openapi`). HTTP/1.1; never panics on bad input.
//!
//! @spec projects/vat/tech-design/logic/built-in-http-mock-record-replay-proxy.md#logic

pub mod ca;
pub mod cassette;
pub mod stub;

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde_json::json;

use ca::CaStore;
use cassette::{Cassettes, Recording};
use stub::Registry;

use crate::emulator::openapi::{OpenApiSpec, Registration, SpecRegistry};

type BoxBody = http_body_util::combinators::BoxBody<Bytes, Infallible>;

struct Proxy {
    ca: CaStore,
    stubs: Registry,
    openapi: SpecRegistry,
    cassettes: Cassettes,
    client: reqwest::Client,
    /// Host-routing table (bare host -> local base URL). Resolved BEFORE
    /// stub/openapi/cassette/forward: a matched host is proxied to the local
    /// emulator and the response returned verbatim, never recorded. Seeded from
    /// `serve`'s `routes` and mutated at runtime via `/__admin/routes`.
    routes: RwLock<HashMap<String, String>>,
}

fn full(status: StatusCode, headers: Vec<(String, String)>, body: Bytes) -> Response<BoxBody> {
    let mut resp = Response::builder().status(status);
    for (k, v) in headers {
        resp = resp.header(k, v);
    }
    resp.body(Full::new(body).boxed())
        .unwrap_or_else(|_| Response::new(Full::new(Bytes::from_static(b"")).boxed()))
}

fn json_resp(status: StatusCode, v: serde_json::Value) -> Response<BoxBody> {
    full(
        status,
        vec![("content-type".into(), "application/json".into())],
        Bytes::from(v.to_string()),
    )
}

/// Serve the HTTP mock proxy until the process is killed. `routes` seeds the
/// host-routing table (`(host, local base URL)` pairs).
pub async fn serve(
    host_port: &str,
    ca_path: &str,
    cassette_dir: &str,
    routes: &[(String, String)],
) -> Result<()> {
    let ca = CaStore::generate().context("mint CA")?;
    // Write the CA pem so vat can export it as the runner's trust bundle.
    std::fs::write(ca_path, ca.ca_pem()).with_context(|| format!("write CA pem {ca_path}"))?;

    let proxy = Arc::new(Proxy {
        ca,
        stubs: Registry::default(),
        openapi: SpecRegistry::default(),
        cassettes: Cassettes::new(cassette_dir),
        client: reqwest::Client::builder()
            .danger_accept_invalid_certs(true) // upstream TLS is recorded, not verified
            .build()
            .context("build upstream client")?,
        routes: RwLock::new(routes.iter().cloned().collect()),
    });

    let listener = tokio::net::TcpListener::bind(host_port)
        .await
        .with_context(|| format!("bind http-mock proxy on {host_port}"))?;
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(pair) => pair,
            Err(_) => continue,
        };
        let proxy = proxy.clone();
        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let svc = service_fn(move |req| {
                let proxy = proxy.clone();
                async move { Ok::<_, Infallible>(proxy.route(req).await) }
            });
            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, svc)
                .with_upgrades()
                .await;
        });
    }
}

impl Proxy {
    /// Top-level routing by the request target form.
    async fn route(self: Arc<Self>, req: Request<Incoming>) -> Response<BoxBody> {
        // CONNECT host:port → MITM the tunnel for HTTPS.
        if req.method() == Method::CONNECT {
            return self.handle_connect(req);
        }
        // Admin control API (origin-form direct request to the proxy port).
        if req.uri().scheme().is_none() && req.uri().path().starts_with("/__admin") {
            return self.handle_admin(req).await;
        }
        // Absolute-form forward proxy request (plain HTTP).
        let scheme = req.uri().scheme_str().unwrap_or("http").to_string();
        let authority = req
            .uri()
            .authority()
            .map(|a| a.as_str().to_string())
            .unwrap_or_default();
        self.handle(&scheme, &authority, req).await
    }

    /// CONNECT: accept, upgrade, terminate TLS with a CA-signed leaf, then serve
    /// HTTP/1 over the decrypted stream (every request is to `host`).
    fn handle_connect(self: Arc<Self>, req: Request<Incoming>) -> Response<BoxBody> {
        let authority = req
            .uri()
            .authority()
            .map(|a| a.as_str().to_string())
            .unwrap_or_default();
        let host = authority
            .split(':')
            .next()
            .unwrap_or(&authority)
            .to_string();
        let server_config = match self.ca.server_config(&host) {
            Ok(cfg) => cfg,
            Err(_) => return full(StatusCode::INTERNAL_SERVER_ERROR, vec![], Bytes::new()),
        };
        let proxy = self.clone();
        tokio::spawn(async move {
            let Ok(upgraded) = hyper::upgrade::on(req).await else {
                return;
            };
            let acceptor = tokio_rustls::TlsAcceptor::from(server_config);
            let Ok(tls) = acceptor.accept(TokioIo::new(upgraded)).await else {
                return;
            };
            let io = TokioIo::new(tls);
            let svc = service_fn(move |r| {
                let proxy = proxy.clone();
                let authority = authority.clone();
                async move { Ok::<_, Infallible>(proxy.handle("https", &authority, r).await) }
            });
            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, svc)
                .await;
        });
        // 200 to the CONNECT so the client proceeds to TLS over the tunnel.
        Response::new(Full::new(Bytes::new()).boxed())
    }

    /// Core handler: route > stub > openapi > cassette replay > forward-and-record.
    /// `authority` is `host[:port]`; stub matching uses the bare hostname, while
    /// the upstream URL and cassette key use the full authority.
    async fn handle(
        &self,
        scheme: &str,
        authority: &str,
        req: Request<Incoming>,
    ) -> Response<BoxBody> {
        let host = authority.split(':').next().unwrap_or(authority);
        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let path_and_query = req
            .uri()
            .path_and_query()
            .map(|pq| pq.as_str().to_string())
            .unwrap_or_else(|| path.clone());
        let req_headers: Vec<(String, String)> = req
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
            .collect();
        let body = req
            .into_body()
            .collect()
            .await
            .map(|c| c.to_bytes())
            .unwrap_or_default();

        // 0) Host route wins: proxy to the local target verbatim, never record.
        let route = self.routes.read().ok().and_then(|r| r.get(host).cloned());
        if let Some(base) = route {
            let url = format!("{}{path_and_query}", base.trim_end_matches('/'));
            return match self.send_upstream(&method, &url, &req_headers, &body).await {
                Ok((status, headers, bytes)) => full(
                    StatusCode::from_u16(status).unwrap_or(StatusCode::OK),
                    headers,
                    bytes,
                ),
                Err(e) => json_resp(
                    StatusCode::BAD_GATEWAY,
                    json!({ "error": format!("route {host} -> {base}: {e}") }),
                ),
            };
        }

        // 1) Stub wins (matched on bare hostname).
        if let Some(s) = self.stubs.find(&method, host, &path) {
            let headers = s.headers.into_iter().collect();
            return full(
                StatusCode::from_u16(s.status).unwrap_or(StatusCode::OK),
                headers,
                Bytes::from(s.body.into_bytes()),
            );
        }

        // 2) A registered OpenAPI spec answers the operation from the contract.
        if let Some(r) = self.openapi.respond(host, &method, &path) {
            return full(
                StatusCode::from_u16(r.status).unwrap_or(StatusCode::OK),
                vec![("content-type".into(), r.content_type)],
                Bytes::from(r.body),
            );
        }

        // 3) Cassette replay (keyed on the full authority).
        let key = Cassettes::key(&method, authority, &path_and_query, &body);
        if let Some(rec) = self.cassettes.get(&key) {
            return full(
                StatusCode::from_u16(rec.status).unwrap_or(StatusCode::OK),
                rec.headers.clone(),
                Bytes::from(rec.body()),
            );
        }

        // 4) Forward to the real upstream and record (auto mode).
        let url = format!("{scheme}://{authority}{path_and_query}");
        match self.send_upstream(&method, &url, &req_headers, &body).await {
            Ok((status, headers, bytes)) => {
                self.cassettes
                    .put(&key, &Recording::new(status, headers.clone(), &bytes));
                full(
                    StatusCode::from_u16(status).unwrap_or(StatusCode::OK),
                    headers,
                    bytes,
                )
            }
            Err(e) => json_resp(
                StatusCode::BAD_GATEWAY,
                json!({ "error": format!("upstream {url}: {e}") }),
            ),
        }
    }

    /// Send `body` to `url` with `method` + caller headers (dropping hop-by-hop /
    /// proxy headers) and return `(status, response headers, body)`. Shared by the
    /// route path (verbatim, no recording) and the forward path (records).
    async fn send_upstream(
        &self,
        method: &str,
        url: &str,
        req_headers: &[(String, String)],
        body: &Bytes,
    ) -> std::result::Result<(u16, Vec<(String, String)>, Bytes), String> {
        let m = reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET);
        let mut rb = self.client.request(m, url);
        for (k, v) in req_headers {
            let lk = k.to_ascii_lowercase();
            if lk == "host" || lk == "proxy-connection" || lk == "connection" {
                continue;
            }
            rb = rb.header(k, v);
        }
        if !body.is_empty() {
            rb = rb.body(body.to_vec());
        }
        match rb.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let headers: Vec<(String, String)> = resp
                    .headers()
                    .iter()
                    .filter(|(k, _)| {
                        let lk = k.as_str().to_ascii_lowercase();
                        lk != "transfer-encoding" && lk != "connection" && lk != "content-length"
                    })
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
                    .collect();
                let bytes = resp.bytes().await.unwrap_or_default();
                Ok((status, headers, bytes))
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Control API: register/clear stubs, list recordings.
    async fn handle_admin(&self, req: Request<Incoming>) -> Response<BoxBody> {
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let body = req
            .into_body()
            .collect()
            .await
            .map(|c| c.to_bytes())
            .unwrap_or_default();
        match (method, path.as_str()) {
            (Method::POST, "/__admin/stubs") => match serde_json::from_slice::<stub::Stub>(&body) {
                Ok(s) => {
                    self.stubs.add(s);
                    json_resp(StatusCode::OK, json!({ "ok": true }))
                }
                Err(e) => json_resp(StatusCode::BAD_REQUEST, json!({ "error": e.to_string() })),
            },
            (Method::DELETE, "/__admin/stubs") => {
                self.stubs.clear();
                json_resp(StatusCode::OK, json!({ "ok": true }))
            }
            (Method::POST, "/__admin/openapi") => {
                match serde_json::from_slice::<Registration>(&body) {
                    Ok(reg) => match OpenApiSpec::from_str(&reg.spec) {
                        Ok(spec) => {
                            self.openapi.add(reg.host, spec);
                            json_resp(StatusCode::OK, json!({ "ok": true }))
                        }
                        Err(e) => {
                            json_resp(StatusCode::BAD_REQUEST, json!({ "error": e.to_string() }))
                        }
                    },
                    Err(e) => json_resp(StatusCode::BAD_REQUEST, json!({ "error": e.to_string() })),
                }
            }
            (Method::DELETE, "/__admin/openapi") => {
                self.openapi.clear();
                json_resp(StatusCode::OK, json!({ "ok": true }))
            }
            (Method::POST, "/__admin/routes") => {
                // Accept a single {host,target} object or an array of them.
                let parsed: std::result::Result<Vec<(String, String)>, String> =
                    match serde_json::from_slice::<serde_json::Value>(&body) {
                        Ok(v) => {
                            let items = if v.is_array() {
                                v.as_array().cloned().unwrap_or_default()
                            } else {
                                vec![v]
                            };
                            items
                                .into_iter()
                                .map(|it| {
                                    let host =
                                        it.get("host").and_then(|h| h.as_str()).map(String::from);
                                    let target =
                                        it.get("target").and_then(|t| t.as_str()).map(String::from);
                                    match (host, target) {
                                        (Some(h), Some(t)) if !h.is_empty() && !t.is_empty() => {
                                            Ok((h, t))
                                        }
                                        _ => Err("each route needs non-empty host + target".into()),
                                    }
                                })
                                .collect()
                        }
                        Err(e) => Err(e.to_string()),
                    };
                match parsed {
                    Ok(pairs) => {
                        if let Ok(mut routes) = self.routes.write() {
                            for (h, t) in pairs {
                                routes.insert(h, t);
                            }
                        }
                        json_resp(StatusCode::OK, json!({ "ok": true }))
                    }
                    Err(e) => json_resp(StatusCode::BAD_REQUEST, json!({ "error": e })),
                }
            }
            (Method::DELETE, "/__admin/routes") => {
                if let Ok(mut routes) = self.routes.write() {
                    routes.clear();
                }
                json_resp(StatusCode::OK, json!({ "ok": true }))
            }
            (Method::GET, "/__admin/recordings") => {
                json_resp(StatusCode::OK, json!({ "keys": self.cassettes.keys() }))
            }
            _ => json_resp(
                StatusCode::NOT_FOUND,
                json!({ "error": "unknown admin route" }),
            ),
        }
    }
}
// CODEGEN-END
