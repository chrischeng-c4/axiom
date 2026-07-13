// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#e2e-test
// <HANDWRITE gap="missing-generator:logic:pgpool-admin-plane" tracker="#1290" reason="Admin plane needs generator primitives that do not exist yet.">
//! End-to-end coverage of the served admin HTTP plane (AC1-AC4), spawning
//! the real `pgpool serve` binary rather than exercising `build_router` in
//! isolation, per the TD E2E Test section. Follows the repo's "real
//! services over mocks, skip gracefully" convention -- see
//! `apps/pgpool/CLAUDE.md`/root `CLAUDE.md` Testing section -- and mirrors
//! `tests/session_proxy.rs`'s real-subprocess-spawn pattern.
//!
//! `BackendPool::new()` never eagerly connects (confirmed by reading
//! `src/pool/backend_pool.rs`), so AC1/AC3 below point `--backend-host`/
//! `--backend-port` at an unreachable placeholder rather than needing a
//! real Postgres -- nothing dials the backend unless a real frontend client
//! actually connects, which neither of those two tests do. AC2/AC4 exercise
//! a genuine in-flight transaction and real gauge movement, so they gate on
//! `real_backend_ready()` like `tests/pool_modes.rs`/`tests/session_proxy.rs`.

use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

/// Confirms the local Postgres is reachable, the current OS user can log in
/// via `trust` auth against the `postgres` database, and a trivial query
/// round-trips. Duplicated from `tests/pool_modes.rs`/`tests/session_proxy.rs`
/// rather than shared -- each `tests/*.rs` file compiles as an independent
/// crate, per this repo's stated convention.
async fn real_backend_ready() -> Option<(SocketAddr, String)> {
    let addr: SocketAddr = "127.0.0.1:5432".parse().ok()?;
    let user = backend_user();
    let dsn = format!(
        "host={} port={} user={} dbname=postgres connect_timeout=2",
        addr.ip(),
        addr.port(),
        user
    );
    let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .ok()?;
    tokio::spawn(connection);
    client.simple_query("SELECT 1").await.ok()?;
    Some((addr, user))
}

fn backend_user() -> String {
    std::env::var("USER").unwrap_or_else(|_| "postgres".to_string())
}

fn proxy_dsn(proxy_addr: SocketAddr, user: &str) -> String {
    format!(
        "host={} port={} user={} dbname=postgres connect_timeout=5",
        proxy_addr.ip(),
        proxy_addr.port(),
        user
    )
}

/// Runs `sql` via the simple query protocol (tag `'Q'`) and returns the
/// first row's `column`, parsed as `i32`. See `tests/session_proxy.rs`'s
/// helper of the same name for why the simple query protocol is used
/// instead of tokio-postgres's default extended query protocol.
async fn simple_query_i32(client: &tokio_postgres::Client, sql: &str, column: &str) -> i32 {
    let messages = client
        .simple_query(sql)
        .await
        .expect("simple-query round-trip through pgpool");
    for message in messages {
        if let tokio_postgres::SimpleQueryMessage::Row(row) = message {
            if let Some(value) = row.get(column) {
                return value.parse().expect("column value parses as i32");
            }
        }
    }
    panic!("simple query {sql:?} returned no row for column {column:?}");
}

/// A real spawned `pgpool serve` subprocess, with both the frontend and
/// admin plane addresses parsed from its startup stdout lines (the exact
/// `"pgpool serve: listening on <addr>"` / `"pgpool serve: admin plane on
/// <addr>"` contract `src/bin/pgpool.rs::serve` prints).
struct ServeProcess {
    child: tokio::process::Child,
    frontend_addr: SocketAddr,
    admin_addr: SocketAddr,
}

impl ServeProcess {
    /// Spawns `pgpool serve` bound to ephemeral ports against `backend_addr`,
    /// plus any `extra_args` (e.g. `--drain-timeout-ms`), and blocks until
    /// both startup lines are observed.
    async fn spawn(backend_addr: SocketAddr, extra_args: &[&str]) -> Self {
        let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_pgpool"))
            .arg("serve")
            .args(["--backend-host", &backend_addr.ip().to_string()])
            .args(["--backend-port", &backend_addr.port().to_string()])
            .args(["--bind", "127.0.0.1:0"])
            .args(["--admin-bind", "127.0.0.1:0"])
            .args(extra_args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .expect("spawn `pgpool serve` subprocess");

        let stdout = child.stdout.take().expect("child stdout piped");
        let mut lines = tokio::io::BufReader::new(stdout).lines();

        let mut frontend_addr = None;
        let mut admin_addr = None;
        while frontend_addr.is_none() || admin_addr.is_none() {
            let line = tokio::time::timeout(Duration::from_secs(5), lines.next_line())
                .await
                .expect("pgpool serve prints its startup lines before timeout")
                .expect("read child stdout")
                .expect("startup line present before stdout closes");
            let line = line.trim();
            if let Some(addr) = line.strip_prefix("pgpool serve: listening on ") {
                frontend_addr = Some(addr.parse().expect("frontend addr parses"));
            } else if let Some(addr) = line.strip_prefix("pgpool serve: admin plane on ") {
                admin_addr = Some(addr.parse().expect("admin addr parses"));
            }
        }

        Self {
            child,
            frontend_addr: frontend_addr.expect("frontend addr observed"),
            admin_addr: admin_addr.expect("admin addr observed"),
        }
    }

    fn admin_url(&self, path: &str) -> String {
        format!("http://{}{path}", self.admin_addr)
    }
}

/// Renders a minimal HTTP/1.1 request with an explicit `Content-Length: 0`
/// and `Connection: keep-alive`, for the raw-socket pipelining
/// `drain_flips_readyz_and_process_exits_cleanly` needs (see that test's
/// doc comment for why plain sequential `reqwest` calls can't observe the
/// flip: the admin listener's accept loop stops taking new connections the
/// instant drain starts, and hyper-util's `GracefulShutdown` closes idle
/// keep-alive connections almost immediately too).
fn http_request(method: &str, path: &str, host: SocketAddr) -> String {
    format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n")
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Parses exactly ONE HTTP/1.1 response (status code + body) out of `buf`,
/// reading more bytes from `stream` as needed, and drains the consumed
/// bytes from `buf` so a following call parses the NEXT pipelined
/// response out of whatever remains. Assumes a `Content-Length`-framed
/// body (true for every admin-plane response here; none are chunked).
async fn read_http_response(
    stream: &mut tokio::net::TcpStream,
    buf: &mut Vec<u8>,
) -> (u16, String) {
    loop {
        if let Some(header_end) = find_subslice(buf, b"\r\n\r\n") {
            let header_str = String::from_utf8_lossy(&buf[..header_end]).into_owned();
            let mut lines = header_str.split("\r\n");
            let status_line = lines.next().expect("status line present");
            let status: u16 = status_line
                .split_whitespace()
                .nth(1)
                .expect("status code present")
                .parse()
                .expect("status code is numeric");
            let content_length: usize = lines
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.trim()
                        .eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().to_string())
                })
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            let body_end = header_end + 4 + content_length;
            if buf.len() >= body_end {
                let body = String::from_utf8_lossy(&buf[header_end + 4..body_end]).into_owned();
                buf.drain(..body_end);
                return (status, body);
            }
        }
        let mut chunk = [0u8; 4096];
        let n = stream
            .read(&mut chunk)
            .await
            .expect("read more response bytes");
        assert!(n > 0, "connection closed before a full response arrived");
        buf.extend_from_slice(&chunk[..n]);
    }
}

/// A placeholder backend address nothing in AC1/AC3 ever dials --
/// `BackendPool::new()` doesn't eagerly connect, so the admin plane binds
/// and serves fine regardless of backend reachability.
fn unreachable_backend() -> SocketAddr {
    "127.0.0.1:1".parse().expect("valid socket address")
}

/// verify: admin::all_routes_respond_on_h2c_and_http1 (AC1, R1)
///
/// Spawns two SEPARATE `pgpool serve` subprocesses -- one driven entirely
/// over h2c, one entirely over HTTP/1.1 -- each issuing `POST /drain` as
/// its own LAST call. Two processes avoid a race between the two protocol
/// clients contending for a single process's post-drain listener state
/// (`h2c::server::serve_with_options`'s accept loop stops taking new
/// connections the instant its shutdown future resolves).
#[tokio::test]
async fn all_routes_respond_on_h2c_and_http1() {
    let get_routes: Vec<String> = pgpool::admin::ADMIN_ROUTES
        .iter()
        .filter(|(method, _)| *method == "GET")
        .map(|(_, path)| path.replace("{pool}", "default"))
        .collect();

    // h2c-driven process.
    {
        let mut serve = ServeProcess::spawn(unreachable_backend(), &[]).await;
        let client = h2c::h2c_client().expect("build h2c client");
        for path in &get_routes {
            let response = client
                .get(serve.admin_url(path))
                .send()
                .await
                .unwrap_or_else(|err| panic!("h2c GET {path} failed: {err}"));
            assert_ne!(
                response.status(),
                reqwest::StatusCode::NOT_FOUND,
                "h2c GET {path} must be routed"
            );
        }
        let response = client
            .post(serve.admin_url("/drain"))
            .send()
            .await
            .expect("h2c POST /drain");
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let body: serde_json::Value = response.json().await.expect("drain response is JSON");
        assert_eq!(body["draining"], true);

        let _ = serve.child.start_kill();
        let _ = serve.child.wait().await;
    }

    // HTTP/1.1-driven process.
    {
        let mut serve = ServeProcess::spawn(unreachable_backend(), &[]).await;
        let client = reqwest::Client::builder()
            .http1_only()
            .build()
            .expect("build http1 client");
        for path in &get_routes {
            let response = client
                .get(serve.admin_url(path))
                .send()
                .await
                .unwrap_or_else(|err| panic!("http1 GET {path} failed: {err}"));
            assert_ne!(
                response.status(),
                reqwest::StatusCode::NOT_FOUND,
                "http1 GET {path} must be routed"
            );
        }
        let response = client
            .post(serve.admin_url("/drain"))
            .send()
            .await
            .expect("http1 POST /drain");
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let body: serde_json::Value = response.json().await.expect("drain response is JSON");
        assert_eq!(body["draining"], true);

        let _ = serve.child.start_kill();
        let _ = serve.child.wait().await;
    }
}

/// verify: admin::drain_flips_readyz_and_process_exits_cleanly (AC2, R2)
///
/// Opens a real transaction (`BEGIN`) through the frontend, drives
/// `POST /drain` while it's still open, confirms `/readyz` flips to 503
/// immediately, lets the transaction run its remaining queries and commit
/// normally, then confirms the process exits with a clean status within
/// the (short, test-configured) drain timeout bound.
///
/// The `POST /drain` + `GET /readyz` pair is pipelined as raw bytes over
/// ONE already-open, already-warm TCP connection (both requests written in
/// a single `write_all` before either response is read) rather than issued
/// as two sequential `reqwest` calls. This is deliberate: the admin
/// listener's accept loop stops taking brand-new connections the very
/// instant drain starts (confirmed by reading `libs/h2c/src/server.rs`),
/// and hyper-util's `GracefulShutdown` closes idle keep-alive connections
/// essentially immediately too -- so a *second*, separately-issued request
/// (whether on a fresh connection or a reused idle one) races the shutdown
/// machinery and reliably loses. Pipelining ensures the `/readyz` bytes are
/// already sitting in the connection's read buffer, as part of the same
/// "currently mid-request" window as `/drain`, before any shutdown
/// notification has a chance to close it out from under us.
#[tokio::test]
async fn drain_flips_readyz_and_process_exits_cleanly() {
    let Some((backend_addr, user)) = real_backend_ready().await else {
        eprintln!(
            "skipping drain_flips_readyz_and_process_exits_cleanly: \
             no reachable local Postgres at 127.0.0.1:5432 for user {:?}",
            backend_user()
        );
        return;
    };

    let mut serve = ServeProcess::spawn(
        backend_addr,
        &[
            "--drain-timeout-ms",
            "3000",
            "--admin-drain-timeout-ms",
            "3000",
        ],
    )
    .await;

    let mut admin_stream = tokio::net::TcpStream::connect(serve.admin_addr)
        .await
        .expect("connect raw admin socket");
    let mut admin_buf: Vec<u8> = Vec::new();

    // Before draining: ready.
    admin_stream
        .write_all(http_request("GET", "/readyz", serve.admin_addr).as_bytes())
        .await
        .expect("write readyz-before request");
    let (status, _) = read_http_response(&mut admin_stream, &mut admin_buf).await;
    assert_eq!(status, 200, "readyz must be 200 before drain starts");

    let dsn = proxy_dsn(serve.frontend_addr, &user);
    let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("client admits");
    let connection_task = tokio::spawn(connection);

    // Open a real in-flight transaction before draining.
    client
        .simple_query("BEGIN")
        .await
        .expect("BEGIN round-trips through pgpool");

    // Pipelined: both requests written in one shot on the SAME already-open
    // connection, before either response is read.
    let pipelined = format!(
        "{}{}",
        http_request("POST", "/drain", serve.admin_addr),
        http_request("GET", "/readyz", serve.admin_addr)
    );
    admin_stream
        .write_all(pipelined.as_bytes())
        .await
        .expect("write pipelined drain+readyz request");

    let (drain_status, drain_body) = read_http_response(&mut admin_stream, &mut admin_buf).await;
    assert_eq!(drain_status, 200);
    let drain_json: serde_json::Value =
        serde_json::from_str(&drain_body).expect("drain response is JSON");
    assert_eq!(drain_json["draining"], true);

    let (readyz_status, readyz_body) = read_http_response(&mut admin_stream, &mut admin_buf).await;
    assert_eq!(
        readyz_status, 503,
        "readyz must flip to 503 once drain starts, got body: {readyz_body}"
    );
    assert!(readyz_body.contains("\"status\":\"draining\""));

    // The in-flight transaction must still be allowed to run its remaining
    // queries and commit normally, undisturbed by the drain flip.
    let value = simple_query_i32(&client, "SELECT 1 AS one", "one").await;
    assert_eq!(value, 1);
    client
        .simple_query("COMMIT")
        .await
        .expect("COMMIT round-trips through pgpool even while draining");

    drop(client);
    connection_task
        .await
        .expect("connection task joined")
        .expect("connection driver ended cleanly");

    let status = tokio::time::timeout(Duration::from_secs(10), serve.child.wait())
        .await
        .expect("pgpool serve exits within the drain timeout after the session finishes")
        .expect("wait on pgpool serve process");
    assert!(
        status.success(),
        "pgpool serve must exit cleanly after draining, got {status:?}"
    );
}

/// verify: admin::served_contract_matches_offline_spec (AC3, R4/R5)
///
/// Diffs the live process's served `/openapi.json` and route set against
/// the offline `pgpool::spec` inventory: byte-for-byte `serde_json::Value`
/// equality for the OpenAPI document, and route-set coverage for every
/// entry in `spec::routes_json()`'s `"routes"` array (excluding the
/// separate `"tcp"` array, which is the Postgres wire-protocol frontend
/// inventory, not an HTTP route).
#[tokio::test]
async fn served_contract_matches_offline_spec() {
    let mut serve = ServeProcess::spawn(unreachable_backend(), &[]).await;
    let client = reqwest::Client::new();

    let response = client
        .get(serve.admin_url("/openapi.json"))
        .send()
        .await
        .expect("GET /openapi.json");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let served_openapi: serde_json::Value =
        response.json().await.expect("openapi response is JSON");
    assert_eq!(
        served_openapi,
        pgpool::spec::openapi(),
        "served /openapi.json must equal pgpool::spec::openapi() exactly"
    );

    let offline: serde_json::Value =
        serde_json::from_str(&pgpool::spec::routes_json()).expect("offline routes_json parses");
    let offline_routes: Vec<(String, String)> = offline["routes"]
        .as_array()
        .expect("routes is an array")
        .iter()
        .map(|route| {
            (
                route["method"].as_str().unwrap().to_string(),
                route["path"].as_str().unwrap().to_string(),
            )
        })
        .collect();

    let served_routes: Vec<(String, String)> = pgpool::admin::ADMIN_ROUTES
        .iter()
        .map(|(m, p)| (m.to_string(), p.to_string()))
        .collect();
    assert_eq!(
        offline_routes, served_routes,
        "served route set must equal the offline routes_json() inventory exactly"
    );

    for (method, path) in &offline_routes {
        if method == "POST" {
            continue; // POST /drain exercised last, below.
        }
        let concrete = path.replace("{pool}", "default");
        let response = client
            .get(serve.admin_url(&concrete))
            .send()
            .await
            .unwrap_or_else(|err| panic!("GET {concrete} failed: {err}"));
        assert_ne!(
            response.status(),
            reqwest::StatusCode::NOT_FOUND,
            "{method} {path} from the offline inventory must be routed"
        );
    }

    // A bogus path must NOT be routed, confirming the comparison above is
    // meaningful (routes are actually being distinguished, not everything
    // accidentally 200ing).
    let response = client
        .get(serve.admin_url("/not-a-real-route"))
        .send()
        .await
        .expect("GET /not-a-real-route");
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);

    let response = client
        .post(serve.admin_url("/drain"))
        .send()
        .await
        .expect("POST /drain");
    assert_ne!(response.status(), reqwest::StatusCode::NOT_FOUND);

    let _ = serve.child.start_kill();
    let _ = serve.child.wait().await;
}

/// verify: admin::metrics_exposes_prometheus_pool_gauges (AC4, R1/R3)
///
/// Scrapes `/metrics` before, during, and after a real transaction to
/// confirm the Prometheus gauges genuinely track live pool state (not just
/// static placeholders), in scrapeable Prometheus text-exposition format.
#[tokio::test]
async fn metrics_exposes_prometheus_pool_gauges() {
    let Some((backend_addr, user)) = real_backend_ready().await else {
        eprintln!(
            "skipping metrics_exposes_prometheus_pool_gauges: \
             no reachable local Postgres at 127.0.0.1:5432 for user {:?}",
            backend_user()
        );
        return;
    };

    let serve = ServeProcess::spawn(backend_addr, &[]).await;
    let admin_client = reqwest::Client::builder()
        .http1_only()
        .pool_max_idle_per_host(0)
        .build()
        .expect("build admin http client");

    let scrape = |url: String| {
        let client = admin_client.clone();
        async move {
            let response = client.get(url).send().await.expect("GET /metrics");
            assert_eq!(response.status(), reqwest::StatusCode::OK);
            assert_eq!(
                response
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .expect("content-type header present"),
                "text/plain;version=0.0.4"
            );
            response.text().await.expect("metrics body is text")
        }
    };

    // Baseline: no client connected yet.
    let baseline = scrape(serve.admin_url("/metrics")).await;
    assert!(baseline.contains("pgpool_frontend_active{pool=\"default\"} 0"));
    assert!(baseline.contains("pgpool_backend_active{pool=\"default\"} 0"));
    assert!(baseline.contains("# TYPE pgpool_frontend_active gauge"));
    assert!(baseline.contains("# TYPE pgpool_backend_active gauge"));
    assert!(baseline.contains("# TYPE pgpool_backend_idle gauge"));

    let dsn = proxy_dsn(serve.frontend_addr, &user);
    let (client, connection) = tokio_postgres::connect(&dsn, tokio_postgres::NoTls)
        .await
        .expect("client admits");
    let connection_task = tokio::spawn(connection);

    // Open a transaction: the per-transaction backend lease stays held
    // (not returned to idle) until COMMIT, since the backend only reports
    // ReadyForQuery(Idle) -- not InTransaction -- once the transaction
    // ends.
    client
        .simple_query("BEGIN")
        .await
        .expect("BEGIN round-trips through pgpool");

    // Poll briefly: the client sees BEGIN's ReadyForQuery before the
    // proxy's own admission/lease bookkeeping is necessarily visible to a
    // concurrent /metrics scrape (same client-visible-before-accounting
    // race as tests/pool_modes.rs's `wait_for_stats`).
    poll_until_metrics(
        &admin_client,
        serve.admin_url("/metrics"),
        "pgpool_backend_active{pool=\"default\"} 1",
    )
    .await;
    let mid_session = scrape(serve.admin_url("/metrics")).await;
    assert!(
        mid_session.contains("pgpool_frontend_active{pool=\"default\"} 1"),
        "frontend_active must reflect the live session, got: {mid_session}"
    );
    assert!(
        mid_session.contains("pgpool_backend_active{pool=\"default\"} 1"),
        "backend_active must reflect the open transaction's held lease, got: {mid_session}"
    );
    assert_ne!(
        baseline, mid_session,
        "gauge values must change between the baseline scrape and the live-session scrape"
    );

    client
        .simple_query("COMMIT")
        .await
        .expect("COMMIT round-trips through pgpool");

    // Poll briefly: the pool's post-relay reset-to-idle runs asynchronously
    // right after the client-visible ReadyForQuery is forwarded, and does so
    // in two steps (leave active accounting, THEN land in idle accounting)
    // with a brief window where neither gauge reflects the connection. Poll
    // on the LATER-settling `backend_idle` condition, not `backend_active`,
    // since idle==1 is the true end state and implies active has already
    // dropped to 0 by then.
    let after_commit = poll_until_metrics(
        &admin_client,
        serve.admin_url("/metrics"),
        "pgpool_backend_idle{pool=\"default\"} 1",
    )
    .await;
    assert!(
        after_commit.contains("pgpool_frontend_active{pool=\"default\"} 1"),
        "session is still open (no Terminate yet), got: {after_commit}"
    );
    assert!(
        after_commit.contains("pgpool_backend_active{pool=\"default\"} 0"),
        "the transaction's held lease must be released after COMMIT, got: {after_commit}"
    );
    assert_ne!(
        mid_session, after_commit,
        "gauge values must change again once the transaction commits"
    );

    drop(client);
    connection_task
        .await
        .expect("connection task joined")
        .expect("connection driver ended cleanly");

    let after_disconnect = poll_until_metrics(
        &admin_client,
        serve.admin_url("/metrics"),
        "pgpool_frontend_active{pool=\"default\"} 0",
    )
    .await;
    assert!(after_disconnect.contains("pgpool_frontend_active{pool=\"default\"} 0"));
}

/// Polls `/metrics` until the body contains `needle` or a 2s deadline
/// elapses (the pool's post-relay state settles asynchronously right after
/// the client-visible response is forwarded).
async fn poll_until_metrics(client: &reqwest::Client, url: String, needle: &str) -> String {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        let body = client
            .get(&url)
            .send()
            .await
            .expect("GET /metrics")
            .text()
            .await
            .expect("metrics body is text");
        if body.contains(needle) || tokio::time::Instant::now() >= deadline {
            return body;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
// </HANDWRITE>
