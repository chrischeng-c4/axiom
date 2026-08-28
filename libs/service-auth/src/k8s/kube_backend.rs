// HANDWRITE-BEGIN gap="missing-generator:logic:k8s-review-transport" tracker="#2869" reason="The kube-rs transport for TokenReview/SubjectAccessReview and its response-shape validation; no generator primitive models a Kubernetes review round trip."
//! The only part of delegated auth that talks to a network.
//!
//! Everything interesting about this file is defensive. A `TokenReview`
//! response is a document the apiserver returns with HTTP 201, and a
//! `SubjectAccessReview` response is the same — a 2xx status says the *request*
//! was accepted, not that it was answered. So each response is translated
//! field by field, and anything that is not a complete answer becomes a
//! [`ReviewError`] rather than a default-valued outcome. A `TokenReviewStatus`
//! deserialized into `Default::default()` would read as `authenticated: false`,
//! which is safe, but a `SubjectAccessReviewStatus` default reads as
//! `allowed: false` *and* `denied: false` — which is "no opinion", not "no".
//! Distinguishing those is the reason this translation is written out instead
//! of derived.

use async_trait::async_trait;
use k8s_openapi::api::authentication::v1::{TokenReview, TokenReviewSpec};
use k8s_openapi::api::authorization::v1::{
    ResourceAttributes as KubeResourceAttributes, SubjectAccessReview, SubjectAccessReviewSpec,
};
use kube::api::{Api, PostParams};
use kube::Client;

use super::principal::ReviewedIdentity;
use super::review::{
    AccessReviewOutcome, ResourceAttributes, ReviewBackend, ReviewError, TokenReviewOutcome,
};

/// A [`ReviewBackend`] backed by a live `kube` client.
pub struct KubeReviewBackend {
    client: Client,
}

impl KubeReviewBackend {
    /// Build a backend from the ambient Kubernetes configuration: the
    /// in-cluster ServiceAccount when running in a pod, the kubeconfig
    /// otherwise.
    ///
    /// Failing here is a startup failure by design. A service that cannot
    /// reach an apiserver cannot authenticate anyone, and starting anyway
    /// means serving 503s while looking healthy.
    pub async fn in_cluster() -> Result<Self, ReviewError> {
        let client = Client::try_default().await.map_err(|error| {
            ReviewError::NotDelegated(format!(
                "no usable Kubernetes client configuration: {error}"
            ))
        })?;
        Ok(Self { client })
    }

    pub fn from_client(client: Client) -> Self {
        Self { client }
    }

    /// Prove the serving identity actually holds both delegation grants, by
    /// issuing the two reviews it will need at request time.
    ///
    /// This runs at startup so that a missing `system:auth-delegator` binding
    /// is a failure to start rather than a 503 on every request an hour later.
    /// A *rejected* probe is a successful probe: what is being tested is
    /// whether the apiserver will answer the question at all, not what the
    /// answer is. `attributes` comes from the caller so this library still
    /// knows nothing about any service's resources.
    pub async fn probe_delegation(
        &self,
        audiences: &[String],
        attributes: &ResourceAttributes,
    ) -> Result<(), ReviewError> {
        self.review_token(DELEGATION_PROBE_TOKEN, audiences).await?;
        self.review_access(
            &ReviewedIdentity {
                username: "system:anonymous".to_string(),
                ..Default::default()
            },
            attributes,
        )
        .await
        .map(|_| ())
    }
}

/// The token the startup probe presents. Not a credential and not valid
/// anywhere; it exists so the probe exercises the real code path instead of a
/// special case, and so the apiserver's audit log names why it was asked.
const DELEGATION_PROBE_TOKEN: &str = "service-auth-delegation-probe";

fn classify(error: kube::Error) -> ReviewError {
    match error {
        kube::Error::Api(response) if response.code == 403 => ReviewError::NotDelegated(format!(
            "the serving identity may not create review resources: {}",
            response.message
        )),
        kube::Error::Api(response) if response.code == 401 => ReviewError::NotDelegated(format!(
            "the serving identity did not authenticate to the apiserver: {}",
            response.message
        )),
        kube::Error::Api(response) => ReviewError::Transport(format!(
            "apiserver returned {}: {}",
            response.code, response.message
        )),
        other => ReviewError::Transport(other.to_string()),
    }
}

#[async_trait]
impl ReviewBackend for KubeReviewBackend {
    async fn review_token(
        &self,
        token: &str,
        audiences: &[String],
    ) -> Result<TokenReviewOutcome, ReviewError> {
        let review = TokenReview {
            spec: TokenReviewSpec {
                token: Some(token.to_string()),
                audiences: (!audiences.is_empty()).then(|| audiences.to_vec()),
            },
            ..Default::default()
        };

        let api: Api<TokenReview> = Api::all(self.client.clone());
        let response = api
            .create(&PostParams::default(), &review)
            .await
            .map_err(classify)?;

        let Some(status) = response.status else {
            return Err(ReviewError::Malformed(
                "TokenReview response carried no status".into(),
            ));
        };

        let user = status.user.unwrap_or_default();
        Ok(TokenReviewOutcome {
            authenticated: status.authenticated.unwrap_or(false),
            identity: ReviewedIdentity {
                username: user.username.unwrap_or_default(),
                uid: user.uid.unwrap_or_default(),
                groups: user.groups.unwrap_or_default(),
                extra: user.extra.unwrap_or_default(),
            },
            audiences: status.audiences.unwrap_or_default(),
            error: status.error,
        })
    }

    async fn review_access(
        &self,
        identity: &ReviewedIdentity,
        attributes: &ResourceAttributes,
    ) -> Result<AccessReviewOutcome, ReviewError> {
        let review = SubjectAccessReview {
            spec: SubjectAccessReviewSpec {
                user: Some(identity.username.clone()),
                uid: (!identity.uid.is_empty()).then(|| identity.uid.clone()),
                groups: (!identity.groups.is_empty()).then(|| identity.groups.clone()),
                extra: (!identity.extra.is_empty()).then(|| identity.extra.clone()),
                resource_attributes: Some(KubeResourceAttributes {
                    group: Some(attributes.group.clone()),
                    namespace: Some(attributes.namespace.clone()),
                    resource: Some(attributes.resource.clone()),
                    name: attributes.name.clone(),
                    verb: Some(attributes.verb.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        let api: Api<SubjectAccessReview> = Api::all(self.client.clone());
        let response = api
            .create(&PostParams::default(), &review)
            .await
            .map_err(classify)?;

        let Some(status) = response.status else {
            return Err(ReviewError::Malformed(
                "SubjectAccessReview response carried no status".into(),
            ));
        };

        Ok(AccessReviewOutcome {
            allowed: status.allowed,
            denied: status.denied.unwrap_or(false),
            reason: status.reason,
            evaluation_error: status.evaluation_error,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use http_body_util::BodyExt;
    use serde_json::{json, Value};
    use tower::service_fn;

    use super::*;

    fn token_review_response() -> Value {
        json!({
            "apiVersion": "authentication.k8s.io/v1",
            "kind": "TokenReview",
            "metadata": {},
            "spec": {},
            "status": {
                "authenticated": true,
                "user": {
                    "username": "system:serviceaccount:apps:api",
                    "uid": "uid-1",
                    "groups": ["system:serviceaccounts"],
                },
                "audiences": [],
            },
        })
    }

    fn recording_backend() -> (KubeReviewBackend, Arc<Mutex<Vec<Value>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let requests = Arc::clone(&seen);
        let service = service_fn(move |request: axum::http::Request<kube::client::Body>| {
            let requests = Arc::clone(&requests);
            async move {
                let body = request.into_body().collect().await.unwrap().to_bytes();
                requests
                    .lock()
                    .unwrap()
                    .push(serde_json::from_slice(&body).unwrap());
                Ok::<_, std::convert::Infallible>(
                    axum::http::Response::builder()
                        .status(201)
                        .header("content-type", "application/json")
                        .body(kube::client::Body::from(
                            serde_json::to_vec(&token_review_response()).unwrap(),
                        ))
                        .unwrap(),
                )
            }
        });
        (
            KubeReviewBackend::from_client(Client::new(service, "default")),
            seen,
        )
    }

    #[tokio::test]
    async fn token_review_omits_only_the_explicit_kubernetes_default_audience() {
        let (backend, seen) = recording_backend();
        backend.review_token("default-token", &[]).await.unwrap();
        backend
            .review_token("managed-token", &["lumen.axiom.dev".into()])
            .await
            .unwrap();

        let requests = seen.lock().unwrap();
        assert!(
            requests[0]["spec"].get("audiences").is_none(),
            "the Kubernetes-default profile must omit spec.audiences"
        );
        assert_eq!(
            requests[1]["spec"]["audiences"],
            json!(["lumen.axiom.dev"]),
            "Managed auth must keep its exact explicit audience"
        );
    }
}
// HANDWRITE-END
