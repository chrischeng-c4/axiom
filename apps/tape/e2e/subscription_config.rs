//! `ackDeadlineSeconds` is subscription configuration, not client folklore
//! (#4014, first row of release Milestone `tape@0.7.0`).
//!
//! The promise in `apps/tape/docs/product/subscriptions.md` § Subscription ack
//! and competing subscribers is that a subscription *owns* its ack deadline.
//! This file pins the configuration half of that promise over the real HTTP
//! surface: create accepts an optional `ackDeadlineSeconds` in `10..=600`,
//! defaults it to `10`, stores it per subscription, echoes it on every read,
//! and refuses everything else with `400` plus a body a client can act on.
//!
//! Lease enforcement — the redelivery timer that eventually *consumes* the
//! deadline — is milestone #117's second row and is deliberately absent here.
//! Nothing below acks, pulls, or waits.
//!
//! ## Why the assertions are shaped the way they are
//!
//! **The refusal cannot be asserted as a bare `400`.** At the tree this file
//! was written against, `SubscriptionCreateRequest`
//! (`apps/tape/src/server.rs:709`) carries `#[serde(deny_unknown_fields)]` and
//! the handler answers `400 bad_request` with serde's `unknown field
//! \`ackDeadlineSeconds\`` message. So a case asserting only "status is 400",
//! or even "the body names the field", is *already green* and would stay green
//! against a server that never learned what the field means. What separates
//! "this server refuses a field it has never heard of" from "this server
//! enforces a range" is the range itself: `assert_range_refusal` requires the
//! body to name both bounds as standalone numeric tokens, which the parser's
//! complaint cannot contain. The same helper serves `9`, `601` and `null`, so
//! the three refusals cannot drift apart.
//!
//! **Stored, not echoed back.** A handler that returned the number it was just
//! handed would satisfy a single-subscription round trip while storing nothing.
//! `an_in_range_ack_deadline_seconds_is_stored_per_subscription_and_echoed`
//! therefore creates two subscriptions with different deadlines on one topic
//! and re-reads both through the independent read routes: a deadline kept on
//! the topic, or in one last-write-wins slot, fails there and nowhere else.
//!
//! **`null` is refused, not coerced.** This is the frozen decision that is
//! easiest to lose: `#[serde(default)] Option<u64>` turns an explicit `null`
//! into `None` into the default `10`, silently, and every assertion about the
//! *status* of that request would still pass if the case only checked that a
//! subscription came back. So the null case checks the other side too — after
//! the refusal the subscription must not exist.
//!
//! **`authorization_is_answered_before_ack_deadline_seconds_is_read` is this
//! file's own negative control, and it is green against the current tree.**
//! `subscription_create` takes `body: axum::body::Bytes` and calls
//! `crate::auth::authorize` before it parses anything, which is why an
//! unauthenticated caller gets `401` rather than a validation verdict. The
//! obvious way to add a validated field is to swap that `Bytes` for a typed
//! `Json<...>` extractor — and axum runs an extractor *before* the handler
//! body, so that one edit would answer `400`/`422` to a caller holding no
//! token at all. That turns the new field into an unauthenticated oracle for
//! which topics and request shapes a node accepts, and it is a regression this
//! work item can introduce without touching a single line of auth code. The
//! case must stay green; `an_authorized_create_still_enforces_the_range` is
//! its red partner, proving the validation is still reachable once past the
//! boundary.

use std::net::SocketAddr;

use serde_json::{json, Value};

use tape::auth::AuthConfig;
use tape::server::{router, AppState};
use tape::TapeJournal;

/// The wire name is Pub/Sub's, per the work item's frozen decisions: camelCase
/// on the JSON surface, not the crate's usual snake_case.
const ACK_FIELD: &str = "ackDeadlineSeconds";

/// The frozen range. Both bounds are inclusive; `DEFAULT` is what a create
/// that omits the field must come back with.
const MIN_ACK_DEADLINE: u64 = 10;
const MAX_ACK_DEADLINE: u64 = 600;
const DEFAULT_ACK_DEADLINE: u64 = 10;

const BODY_LIMIT: usize = 8 * 1024 * 1024;

/// producer holds `write` on the fixture topic, worker holds `read`, and root
/// is a wildcard admin. Mirrors `e2e/service_auth.rs`'s registry so the two
/// files describe one authorization model.
const REGISTRY: &str = r#"{
    "writer-token": {"subject": "producer", "roles": {"acks": "write"}},
    "reader-token": {"subject": "worker", "roles": {"acks": "read"}},
    "admin-token": {"subject": "root", "roles": {"*": "admin"}}
}"#;

// --------------------------------------------------------------------------
// harness
// --------------------------------------------------------------------------

/// Start the real app on a loopback port through the shared service shell's
/// serve loop, exactly as `e2e/http_transport.rs` does. Auth off: the
/// behaviour cases are about the field, not the boundary.
async fn start_server() -> SocketAddr {
    spawn(AppState::new(TapeJournal::default(), None, BODY_LIMIT)).await
}

/// The same server with the shared bearer-token contract in `required` mode,
/// resolved through the real registry-file loader.
async fn start_server_with_auth() -> SocketAddr {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("token-registry.json");
    std::fs::write(&path, REGISTRY).unwrap();
    let cfg = AuthConfig::resolve("required", Some(path.to_str().unwrap()), None).unwrap();
    // Short-lived test process; keep the registry file alive for the config.
    std::mem::forget(dir);
    spawn(AppState::with_auth(
        TapeJournal::default(),
        None,
        cfg,
        BODY_LIMIT,
    ))
    .await
}

async fn spawn(state: AppState) -> SocketAddr {
    let app = router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(service_http::serve(
        listener,
        app,
        std::future::pending::<()>(),
    ));
    addr
}

fn url(addr: SocketAddr, path: &str) -> String {
    format!("http://{addr}{path}")
}

/// One real event on the topic, so every subscription below is a cursor over
/// something rather than a name in an empty map.
async fn seed_topic(client: &reqwest::Client, addr: SocketAddr, topic: &str, token: Option<&str>) {
    let mut req = client
        .post(url(addr, &format!("/topics/{topic}/append")))
        .json(&json!({ "payload": { "seeded": true } }));
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req.send().await.unwrap();
    assert!(
        resp.status().is_success(),
        "seeding topic `{topic}` must succeed, got {} {}",
        resp.status(),
        resp.text().await.unwrap_or_default()
    );
}

struct Answer {
    status: reqwest::StatusCode,
    body: String,
}

impl Answer {
    /// The response body as JSON, or a failure naming what came back instead.
    /// Every route under test answers `application/json`, including refusals.
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or_else(|error| {
            panic!(
                "response body must be JSON ({error}); status {} body {:?}",
                self.status, self.body
            )
        })
    }
}

async fn send(req: reqwest::RequestBuilder) -> Answer {
    let resp = req.send().await.unwrap();
    let status = resp.status();
    let body = resp.text().await.unwrap();
    Answer { status, body }
}

/// `POST /topics/{topic}/subscriptions` with a caller-supplied body, sent as
/// raw bytes so a case can post a shape `serde_json::json!` cannot build.
async fn create_raw(
    client: &reqwest::Client,
    addr: SocketAddr,
    topic: &str,
    body: &str,
    token: Option<&str>,
) -> Answer {
    let mut req = client
        .post(url(addr, &format!("/topics/{topic}/subscriptions")))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body.to_string());
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    send(req).await
}

/// Create with `name` and, when `ack` is `Some`, that exact JSON value for
/// `ackDeadlineSeconds`. `None` omits the key entirely — the two are different
/// requests and the difference is the default's whole contract.
async fn create(
    client: &reqwest::Client,
    addr: SocketAddr,
    topic: &str,
    name: &str,
    ack: Option<&str>,
) -> Answer {
    let body = match ack {
        Some(value) => format!(r#"{{"name":"{name}","{ACK_FIELD}":{value}}}"#),
        None => format!(r#"{{"name":"{name}"}}"#),
    };
    create_raw(client, addr, topic, &body, None).await
}

async fn read(
    client: &reqwest::Client,
    addr: SocketAddr,
    topic: &str,
    name: &str,
    token: Option<&str>,
) -> Answer {
    let mut req = client.get(url(
        addr,
        &format!("/topics/{topic}/subscriptions/{name}"),
    ));
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    send(req).await
}

async fn list(client: &reqwest::Client, addr: SocketAddr, topic: &str) -> Answer {
    send(client.get(url(addr, &format!("/topics/{topic}/subscriptions")))).await
}

// --------------------------------------------------------------------------
// oracles
// --------------------------------------------------------------------------

/// The deadline a response carries, or a failure naming what it carried
/// instead. Absence is the interesting failure and gets its own message: it is
/// how the current tree fails, and "key missing" must not read as "wrong
/// number".
fn ack_deadline(value: &Value, what: &str) -> u64 {
    let field = value.get(ACK_FIELD).unwrap_or_else(|| {
        panic!("{what} must carry `{ACK_FIELD}`; got {value}")
    });
    field.as_u64().unwrap_or_else(|| {
        panic!("{what} `{ACK_FIELD}` must be a JSON integer; got {field}")
    })
}

/// Every run of ASCII digits in `text`, as whole tokens. Tokenising rather
/// than substring-matching is what stops `601` from reading as evidence that a
/// message named the bound `60`, and `100` from standing in for `10`.
fn numeric_tokens(text: &str) -> Vec<&str> {
    text.split(|c: char| !c.is_ascii_digit())
        .filter(|token| !token.is_empty())
        .collect()
}

/// The frozen refusal: `400`, the shared machine-readable envelope, the field
/// by name, and both bounds of the allowed range.
///
/// The last requirement is the load-bearing one. A server that has never heard
/// of `ackDeadlineSeconds` already answers `400` and already repeats the field
/// name back (serde's unknown-field complaint contains it), so those two rows
/// alone describe the tree this file was written against. Naming the range is
/// something only a server that knows the range can do.
fn assert_range_refusal(answer: &Answer, sent: &str) {
    assert_eq!(
        answer.status.as_u16(),
        400,
        "create with `{ACK_FIELD}`: {sent} must be refused with 400; got {} body {:?}",
        answer.status,
        answer.body
    );

    let body = answer.json();
    let object = body
        .as_object()
        .unwrap_or_else(|| panic!("refusal body must be a JSON object; got {body}"));

    let kind = object
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("refusal body must carry a string `error` kind; got {body}"));
    assert!(
        !kind.is_empty() && !kind.contains(char::is_whitespace),
        "`error` must be a machine-stable kind a client can branch on, not prose; got {kind:?}"
    );
    assert!(
        object.contains_key("message"),
        "refusal body must carry `message`; got {body}"
    );

    assert!(
        answer.body.contains(ACK_FIELD),
        "refusal for {sent} must name `{ACK_FIELD}` so the client knows which \
         field to fix; got {:?}",
        answer.body
    );

    let tokens = numeric_tokens(&answer.body);
    let min = MIN_ACK_DEADLINE.to_string();
    let max = MAX_ACK_DEADLINE.to_string();
    assert!(
        tokens.iter().any(|token| *token == min),
        "refusal for {sent} must name the lower bound {min} so the client can \
         choose a legal value without reading the source; got {:?}",
        answer.body
    );
    assert!(
        tokens.iter().any(|token| *token == max),
        "refusal for {sent} must name the upper bound {max}; got {:?}",
        answer.body
    );
}

/// A refused create leaves nothing behind. This is what separates "refused"
/// from "clamped into range and stored", and from "stored, then reported as an
/// error".
async fn assert_absent(client: &reqwest::Client, addr: SocketAddr, topic: &str, name: &str) {
    let answer = read(client, addr, topic, name, None).await;
    assert_eq!(
        answer.status.as_u16(),
        404,
        "a refused create must leave no subscription `{name}`; reading it back \
         gave {} body {:?}",
        answer.status,
        answer.body
    );
}

// --------------------------------------------------------------------------
// behaviour: the default
// --------------------------------------------------------------------------

/// A create that omits the field gets the default, and the default is visible
/// — a subscriber must be able to discover its deadline without being told the
/// number out of band, which is the entire point of making it configuration.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn create_without_ack_deadline_seconds_defaults_to_ten() {
    let addr = start_server().await;
    let client = reqwest::Client::new();
    seed_topic(&client, addr, "acks", None).await;

    let created = create(&client, addr, "acks", "plain", None).await;
    assert_eq!(
        created.status.as_u16(),
        201,
        "create without `{ACK_FIELD}` must still succeed; got {} body {:?}",
        created.status,
        created.body
    );
    assert_eq!(
        ack_deadline(&created.json(), "the create response"),
        DEFAULT_ACK_DEADLINE
    );

    let fetched = read(&client, addr, "acks", "plain", None).await;
    assert_eq!(fetched.status.as_u16(), 200, "read back: {:?}", fetched.body);
    assert_eq!(
        ack_deadline(&fetched.json(), "the read response"),
        DEFAULT_ACK_DEADLINE
    );

    let listed = list(&client, addr, "acks").await;
    let body = listed.json();
    let entry = body["subscriptions"]
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row["name"] == "plain").cloned())
        .unwrap_or_else(|| panic!("list must contain `plain`; got {body}"));
    assert_eq!(
        ack_deadline(&entry, "the list entry"),
        DEFAULT_ACK_DEADLINE
    );
}

// --------------------------------------------------------------------------
// behaviour: the round trip
// --------------------------------------------------------------------------

/// An explicit in-range value is persisted against *that* subscription and
/// echoed on every read route.
///
/// Two subscriptions with different deadlines on one topic, read back in the
/// opposite order they were written. A handler that echoes its own request
/// body, or keeps one deadline per topic, or keeps the last one written,
/// passes a single-subscription round trip and fails here.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_in_range_ack_deadline_seconds_is_stored_per_subscription_and_echoed() {
    let addr = start_server().await;
    let client = reqwest::Client::new();
    seed_topic(&client, addr, "acks", None).await;

    let slow = create(&client, addr, "acks", "slow", Some("300")).await;
    assert_eq!(
        slow.status.as_u16(),
        201,
        "an in-range `{ACK_FIELD}` must be accepted; got {} body {:?}",
        slow.status,
        slow.body
    );
    assert_eq!(ack_deadline(&slow.json(), "the create response"), 300);

    let quick = create(&client, addr, "acks", "quick", Some("45")).await;
    assert_eq!(quick.status.as_u16(), 201, "second create: {:?}", quick.body);
    assert_eq!(ack_deadline(&quick.json(), "the create response"), 45);

    // Reverse order: the second write must not have moved the first.
    let quick_read = read(&client, addr, "acks", "quick", None).await;
    assert_eq!(quick_read.status.as_u16(), 200, "{:?}", quick_read.body);
    assert_eq!(ack_deadline(&quick_read.json(), "`quick` on read"), 45);

    let slow_read = read(&client, addr, "acks", "slow", None).await;
    assert_eq!(slow_read.status.as_u16(), 200, "{:?}", slow_read.body);
    assert_eq!(ack_deadline(&slow_read.json(), "`slow` on read"), 300);

    // And the default is per subscription too, not a topic-wide setting the
    // explicit values overwrote.
    let bare = create(&client, addr, "acks", "bare", None).await;
    assert_eq!(bare.status.as_u16(), 201, "{:?}", bare.body);
    assert_eq!(
        ack_deadline(&bare.json(), "`bare` on create"),
        DEFAULT_ACK_DEADLINE
    );

    let listed = list(&client, addr, "acks").await;
    let body = listed.json();
    let rows = body["subscriptions"]
        .as_array()
        .unwrap_or_else(|| panic!("list must return an array; got {body}"));
    for (name, want) in [("slow", 300_u64), ("quick", 45), ("bare", DEFAULT_ACK_DEADLINE)] {
        let row = rows
            .iter()
            .find(|row| row["name"] == name)
            .unwrap_or_else(|| panic!("list must contain `{name}`; got {body}"));
        assert_eq!(
            ack_deadline(row, &format!("the list entry for `{name}`")),
            want
        );
    }
}

// --------------------------------------------------------------------------
// behaviour: the refusals
// --------------------------------------------------------------------------

/// One below the floor. Refused with the frozen 400 shape, and nothing is
/// created — a deadline quietly clamped up to 10 would be a subscription whose
/// configuration disagrees with what its owner asked for.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ack_deadline_seconds_of_nine_is_refused_with_400() {
    let addr = start_server().await;
    let client = reqwest::Client::new();
    seed_topic(&client, addr, "acks", None).await;

    let answer = create(&client, addr, "acks", "too-eager", Some("9")).await;
    assert_range_refusal(&answer, "9");
    assert_absent(&client, addr, "acks", "too-eager").await;
}

/// One above the ceiling, refused by the same helper as `9`, so the two ends
/// of the range cannot be given different shapes. This is the case the work
/// item's negative control drives red by dropping the upper bound.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ack_deadline_seconds_of_six_hundred_one_is_refused_with_400() {
    let addr = start_server().await;
    let client = reqwest::Client::new();
    seed_topic(&client, addr, "acks", None).await;

    let answer = create(&client, addr, "acks", "too-patient", Some("601")).await;
    assert_range_refusal(&answer, "601");
    assert_absent(&client, addr, "acks", "too-patient").await;
}

/// An explicit `null` is refused like any other out-of-range value.
///
/// The frozen decision names the failure mode directly: `null` must not be
/// coerced. `#[serde(default)] Option<u64>` coerces it — to `None`, to the
/// default — and the caller who sent it believes they configured something.
/// Both halves are asserted, because either one alone can be satisfied by the
/// wrong implementation: the status by a parser that rejects `null` for the
/// wrong reason, the absence by a server that refuses every create.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_null_ack_deadline_seconds_is_refused_and_never_coerced_to_the_default() {
    let addr = start_server().await;
    let client = reqwest::Client::new();
    seed_topic(&client, addr, "acks", None).await;

    let answer = create(&client, addr, "acks", "nulled", Some("null")).await;
    assert_range_refusal(&answer, "null");
    assert_absent(&client, addr, "acks", "nulled").await;

    // The distinguishing half: omitting the key is a different request, and it
    // is the one that means "give me the default".
    let omitted = create(&client, addr, "acks", "omitted", None).await;
    assert_eq!(
        omitted.status.as_u16(),
        201,
        "omitting the key must still mean the default; got {} body {:?}",
        omitted.status,
        omitted.body
    );
    assert_eq!(
        ack_deadline(&omitted.json(), "the create response"),
        DEFAULT_ACK_DEADLINE
    );
}

/// Both bounds are inclusive. Stated separately from the refusals because an
/// off-by-one in either direction — accepting `11..=599` — satisfies every
/// refusal case above while rejecting the two values the promise cites.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_range_bounds_ten_and_six_hundred_are_inclusive() {
    let addr = start_server().await;
    let client = reqwest::Client::new();
    seed_topic(&client, addr, "acks", None).await;

    for (name, value) in [("floor", MIN_ACK_DEADLINE), ("ceiling", MAX_ACK_DEADLINE)] {
        let answer = create(&client, addr, "acks", name, Some(&value.to_string())).await;
        assert_eq!(
            answer.status.as_u16(),
            201,
            "the bound {value} is inclusive and must be accepted; got {} body {:?}",
            answer.status,
            answer.body
        );
        assert_eq!(
            ack_deadline(&answer.json(), "the create response"),
            value
        );

        let fetched = read(&client, addr, "acks", name, None).await;
        assert_eq!(fetched.status.as_u16(), 200, "{:?}", fetched.body);
        assert_eq!(ack_deadline(&fetched.json(), "the read response"), value);
    }
}

// --------------------------------------------------------------------------
// security: input hardening
// --------------------------------------------------------------------------

/// Values that are not an integer in range are refused and create nothing.
///
/// The status is asserted as "a client error", not as one number: the work item
/// freezes `400` for an out-of-range *value*, and a type error is a different
/// answer that this contract deliberately does not invent. What it does fix is
/// the part that matters — the request is never accepted, the server keeps
/// answering afterwards, and no subscription appears. A `u64` overflow, a
/// negative, or a float that silently truncates into range would each be a
/// deadline nobody configured.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn malformed_ack_deadline_seconds_values_are_refused_and_create_nothing() {
    let addr = start_server().await;
    let client = reqwest::Client::new();
    seed_topic(&client, addr, "acks", None).await;

    let malformed = [
        ("negative", "-1"),
        ("negative-in-range-magnitude", "-300"),
        ("string", "\"300\""),
        ("float", "12.5"),
        ("float-in-range", "300.7"),
        ("beyond-u64", "18446744073709551616"),
        ("boolean", "true"),
        ("array", "[300]"),
        ("object", "{\"seconds\":300}"),
    ];

    for (label, value) in malformed {
        let name = format!("bad-{label}");
        let answer = create(&client, addr, "acks", &name, Some(value)).await;
        assert!(
            answer.status.is_client_error(),
            "`{ACK_FIELD}`: {value} ({label}) must be refused as a client error; \
             got {} body {:?}",
            answer.status,
            answer.body
        );
        assert!(
            !answer.status.is_success(),
            "`{ACK_FIELD}`: {value} ({label}) must not be accepted; got {}",
            answer.status
        );
        assert_absent(&client, addr, "acks", &name).await;
    }

    // The server is still serving, and the field still works: a hardening
    // sweep that left the route dead would satisfy every row above.
    let good = create(&client, addr, "acks", "well-formed", Some("42")).await;
    assert_eq!(
        good.status.as_u16(),
        201,
        "a well-formed create after the malformed sweep must succeed; got {} body {:?}",
        good.status,
        good.body
    );
    assert_eq!(ack_deadline(&good.json(), "the create response"), 42);
}

// --------------------------------------------------------------------------
// security: the authorization boundary
// --------------------------------------------------------------------------

/// The authorization boundary answers before `ackDeadlineSeconds` is read.
///
/// **This case is green against the current tree and must stay green.** It is
/// this file's negative control against the most likely way to implement the
/// rest of it: replacing the handler's `body: axum::body::Bytes` with a typed
/// `Json<SubscriptionCreateRequest>` extractor. Axum runs an extractor before
/// the handler body, so that edit moves validation *ahead* of
/// `crate::auth::authorize` and hands an anonymous caller a `400` describing
/// the allowed range — a validation oracle reachable with no token, on a topic
/// the caller has no grant for. Every other case in this file would still be
/// green after that edit.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn authorization_is_answered_before_ack_deadline_seconds_is_read() {
    let addr = start_server_with_auth().await;
    let client = reqwest::Client::new();
    seed_topic(&client, addr, "acks", Some("admin-token")).await;

    // Deliberately out of range: the only reason to answer anything other than
    // 401/403 here is having validated the field before checking the caller.
    let body = format!(r#"{{"name":"probe","{ACK_FIELD}":601}}"#);

    let anonymous = create_raw(&client, addr, "acks", &body, None).await;
    assert_eq!(
        anonymous.status.as_u16(),
        401,
        "a tokenless create must be refused by the auth boundary, not by \
         `{ACK_FIELD}` validation; got {} body {:?}",
        anonymous.status,
        anonymous.body
    );

    let unknown = create_raw(&client, addr, "acks", &body, Some("not-a-real-token")).await;
    assert_eq!(
        unknown.status.as_u16(),
        401,
        "an unknown token must be refused by the auth boundary; got {} body {:?}",
        unknown.status,
        unknown.body
    );

    let read_only = create_raw(&client, addr, "acks", &body, Some("reader-token")).await;
    assert_eq!(
        read_only.status.as_u16(),
        403,
        "a read-only token must be refused by authorization, not by \
         `{ACK_FIELD}` validation; got {} body {:?}",
        read_only.status,
        read_only.body
    );

    // Fail closed: none of the three refused calls may have written anything.
    let fetched = read(&client, addr, "acks", "probe", Some("admin-token")).await;
    assert_eq!(
        fetched.status.as_u16(),
        404,
        "no refused create may leave a subscription behind; got {} body {:?}",
        fetched.status,
        fetched.body
    );
}

/// Past the boundary, the range is still enforced.
///
/// The partner to the case above: an implementation that satisfied the
/// ordering guard by never validating on the authenticated path would be
/// trading one defect for another.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_authorized_create_still_enforces_the_range() {
    let addr = start_server_with_auth().await;
    let client = reqwest::Client::new();
    seed_topic(&client, addr, "acks", Some("admin-token")).await;

    let refused = create_raw(
        &client,
        addr,
        "acks",
        &format!(r#"{{"name":"granted","{ACK_FIELD}":601}}"#),
        Some("writer-token"),
    )
    .await;
    assert_range_refusal(&refused, "601 with a write grant");

    let accepted = create_raw(
        &client,
        addr,
        "acks",
        &format!(r#"{{"name":"granted","{ACK_FIELD}":300}}"#),
        Some("writer-token"),
    )
    .await;
    assert_eq!(
        accepted.status.as_u16(),
        201,
        "an in-range create with a write grant must succeed; got {} body {:?}",
        accepted.status,
        accepted.body
    );
    assert_eq!(
        ack_deadline(&accepted.json(), "the create response"),
        300
    );

    let fetched = read(&client, addr, "acks", "granted", Some("reader-token")).await;
    assert_eq!(fetched.status.as_u16(), 200, "{:?}", fetched.body);
    assert_eq!(ack_deadline(&fetched.json(), "the read response"), 300);
}
