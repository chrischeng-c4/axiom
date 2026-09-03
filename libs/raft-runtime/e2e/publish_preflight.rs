//! Publish preflight contract (#4165).
//!
//! A follower knows that it is not the publish destination before it can rely
//! on a media type or a decodable JSON body. It therefore returns one stable
//! `421` leader hint for malformed direct requests. A fully well-formed request
//! for another group is different: it is rejected as `400` first, so a host
//! does not disclose a leader for a group it does not own.
//!
//! The same ordering must hold through a single-host [`RaftRegistry`]. That
//! route must pass the original headers and body to the shared host preflight,
//! rather than decode the request before the host can make its routing choice.
//! A leader continues to expose Axum's existing input-rejection statuses:
//! missing content type is `415`, invalid JSON syntax is `400`, and an invalid
//! envelope shape is `422`.

use std::sync::Arc;

use raft_runtime::{RaftHost, RaftRegistry};

#[path = "support/cluster.rs"]
mod cluster;
use cluster::{await_leader, bind, cluster};

#[derive(Clone, Copy, Debug)]
enum InvalidPublish {
    MissingContentType,
    Syntax,
    DataShape,
}

impl InvalidPublish {
    fn label(self) -> &'static str {
        match self {
            Self::MissingContentType => "missing Content-Type",
            Self::Syntax => "invalid JSON syntax",
            Self::DataShape => "invalid envelope shape",
        }
    }

    fn leader_status(self) -> reqwest::StatusCode {
        match self {
            Self::MissingContentType => reqwest::StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::Syntax => reqwest::StatusCode::BAD_REQUEST,
            Self::DataShape => reqwest::StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}

fn h2c_client() -> reqwest::Client {
    reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .expect("the h2c client builds")
}

fn publish_url(base_url: &str) -> String {
    format!("{base_url}{}", RaftHost::PUBLISH_PATH)
}

async fn send_invalid_publish(
    client: &reqwest::Client,
    base_url: &str,
    group_id: &str,
    invalid: InvalidPublish,
) -> reqwest::Response {
    let request = match invalid {
        InvalidPublish::MissingContentType => client.post(publish_url(base_url)).body(
            serde_json::to_vec(&serde_json::json!({
                "group_id": group_id,
                "command": [7],
            }))
            .expect("the well-formed request body serializes"),
        ),
        InvalidPublish::Syntax => client
            .post(publish_url(base_url))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(format!(
                r#"{{"group_id":{},"command":["#,
                serde_json::to_string(group_id).expect("the group id serializes")
            )),
        InvalidPublish::DataShape => client.post(publish_url(base_url)).json(&serde_json::json!({
            "group_id": group_id,
            "command": true,
        })),
    };

    request
        .send()
        .await
        .expect("the publish request reaches the h2c router")
}

async fn assert_not_leader(
    response: reqwest::Response,
    expected_leader: u64,
    surface: &str,
    invalid: InvalidPublish,
) {
    let status = response.status();
    let body = response
        .text()
        .await
        .expect("the response body is readable");
    assert_eq!(
        status,
        reqwest::StatusCode::MISDIRECTED_REQUEST,
        "{surface} follower must preflight {} as 421; body: {body}",
        invalid.label(),
    );
    let json: serde_json::Value =
        serde_json::from_str(&body).expect("the follower response is JSON");
    assert_eq!(
        json,
        serde_json::json!({
            "error": "not-leader",
            "leader": expected_leader,
        }),
        "{surface} follower must return exactly one stable leader hint for {}",
        invalid.label(),
    );
}

async fn assert_leader_validation(
    response: reqwest::Response,
    surface: &str,
    invalid: InvalidPublish,
) {
    let status = response.status();
    let body = response
        .text()
        .await
        .expect("the response body is readable");
    assert_eq!(
        status,
        invalid.leader_status(),
        "{surface} leader must retain {} for {}; body: {body}",
        invalid.leader_status(),
        invalid.label(),
    );
}

async fn serve_single_host_registry(host: Arc<RaftHost>) -> (String, tokio::task::JoinHandle<()>) {
    let registry = RaftRegistry::new();
    registry
        .register(host)
        .expect("a fresh registry accepts its one host");
    let (listener, url) = bind().await;
    let router = registry.router();
    let serve = tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let router = router.clone();
                tokio::spawn(async move {
                    let _ = transport_h2c::server::serve_connection(stream, router).await;
                });
            }
        }
    });
    (url, serve)
}

/// The public route name is shared by callers and both router shapes. A foreign
/// envelope must be rejected before a follower reveals its own leader hint.
#[tokio::test]
async fn canonical_publish_path_rejects_foreign_groups_on_direct_hosts() {
    assert_eq!(
        RaftHost::PUBLISH_PATH,
        "/raft/publish",
        "the public publish route must retain its canonical path"
    );

    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("the three-voter cluster elects one leader");
    let follower = (leader + 1) % nodes.len();
    let group_id = nodes[leader].host.group_id().0.clone();
    let foreign_group = format!("{group_id}-foreign");
    let client = h2c_client();

    for (surface, node) in [("leader", leader), ("follower", follower)] {
        let response = client
            .post(publish_url(&nodes[node].url))
            .json(&serde_json::json!({
                "group_id": foreign_group,
                "command": [7],
            }))
            .send()
            .await
            .expect("the well-formed foreign-group request reaches the direct host");
        let status = response.status();
        let body = response
            .text()
            .await
            .expect("the response body is readable");
        assert_eq!(
            status,
            reqwest::StatusCode::BAD_REQUEST,
            "{surface} direct host must reject a well-formed foreign group as 400; body: {body}"
        );
    }
}

/// A follower exposes the same not-leader response before all three validation
/// failures through the direct host and through the registry router.
#[tokio::test]
async fn follower_preflight_precedes_json_rejection_for_direct_host_and_registry() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("the three-voter cluster elects one leader");
    let follower = (leader + 1) % nodes.len();
    let expected_leader = leader as u64;
    let group_id = nodes[follower].host.group_id().0.clone();
    let client = h2c_client();
    let (registry_url, registry_serve) =
        serve_single_host_registry(Arc::clone(&nodes[follower].host)).await;

    for invalid in [
        InvalidPublish::MissingContentType,
        InvalidPublish::Syntax,
        InvalidPublish::DataShape,
    ] {
        let direct = send_invalid_publish(&client, &nodes[follower].url, &group_id, invalid).await;
        assert_not_leader(direct, expected_leader, "direct host", invalid).await;

        let registry = send_invalid_publish(&client, &registry_url, &group_id, invalid).await;
        assert_not_leader(registry, expected_leader, "registry", invalid).await;
    }

    registry_serve.abort();
}

/// A leader does not relax media-type or JSON validation, whether its route is
/// served directly or through the registry router.
#[tokio::test]
async fn leader_retains_json_rejection_statuses_for_direct_host_and_registry() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("the three-voter cluster elects one leader");
    let group_id = nodes[leader].host.group_id().0.clone();
    let client = h2c_client();
    let (registry_url, registry_serve) =
        serve_single_host_registry(Arc::clone(&nodes[leader].host)).await;

    for invalid in [
        InvalidPublish::MissingContentType,
        InvalidPublish::Syntax,
        InvalidPublish::DataShape,
    ] {
        let direct = send_invalid_publish(&client, &nodes[leader].url, &group_id, invalid).await;
        assert_leader_validation(direct, "direct host", invalid).await;

        let registry = send_invalid_publish(&client, &registry_url, &group_id, invalid).await;
        assert_leader_validation(registry, "registry", invalid).await;
    }

    registry_serve.abort();
}
