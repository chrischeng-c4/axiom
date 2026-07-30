// HANDWRITE-BEGIN gap="missing-generator:logic:certificate-cas-requester" tracker="#3110" reason="Own the CA Service request shape and the workload-identity token exchange, kept behind a feature so the rest of the lifecycle can be built and tested with no cloud dependency at all."
//! Asking GCP CA Service for a certificate.
//!
//! Optional, and that is structural rather than tidiness: everything that makes
//! this lifecycle correct — when to renew, what order to publish things in, what
//! to refuse — lives in modules that have never heard of GCP. This one turns a
//! CSR into an HTTP request and a response into a leaf. Under
//! `--no-default-features` it is not compiled, and the lifecycle still builds,
//! still tests, and still issues certificates through
//! [`super::ephemeral::EphemeralIssuer`] (R8).
//!
//! ### Credentials
//!
//! There are none to hold. The only thing this module reads is the projected
//! KSA token the kubelet writes and rotates, which it exchanges at STS for a
//! short-lived federated access token. There is no service-account key, no ADC
//! file, no metadata-server credential — the pool's IAM binding names the
//! workload's `principal://` directly (#3109), so the KSA *is* the identity.
//!
//! ### Retries do not mint duplicates
//!
//! The certificate id is derived from the CSR. A request that timed out after
//! the CA had already signed comes back with the same id, and CA Service
//! returns the existing certificate rather than issuing a second one. Without
//! that, every network hiccup during renewal would leave a stray valid leaf
//! behind — valid, unreferenced, and counted against nothing.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use futures::future::BoxFuture;
use serde_json::{json, Value};

use super::digest::hex_sha256;
use super::issuer::{IssuanceRequest, IssuedMaterial, Issuer, IssuerError, IssuerId};
use super::projection::parse_leaf;

/// The pool a certificate is requested from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaPool {
    pub project: String,
    pub location: String,
    pub pool: String,
}

impl CaPool {
    /// Parse `projects/P/locations/L/caPools/N` — the exact string #3109's
    /// Terraform emits as an output, so the operator's configuration is a copy
    /// rather than four fields someone reassembles by hand.
    pub fn parse(resource: &str) -> Result<Self, IssuerError> {
        let parts: Vec<&str> = resource.split('/').collect();
        match parts.as_slice() {
            ["projects", project, "locations", location, "caPools", pool]
                if !project.is_empty() && !location.is_empty() && !pool.is_empty() =>
            {
                Ok(Self {
                    project: (*project).to_string(),
                    location: (*location).to_string(),
                    pool: (*pool).to_string(),
                })
            }
            _ => Err(IssuerError::Upstream(format!(
                "not a CA pool resource name: {resource}"
            ))),
        }
    }

    pub fn resource(&self) -> String {
        format!(
            "projects/{}/locations/{}/caPools/{}",
            self.project, self.location, self.pool
        )
    }
}

/// Anything that can produce a bearer token for `privateca.googleapis.com`.
pub trait AccessTokenSource: Send + Sync {
    fn token<'a>(&'a self) -> BoxFuture<'a, Result<String, IssuerError>>;
}

/// Exchanges the kubelet's projected KSA token for a federated access token.
///
/// The projected token is re-read on every exchange rather than cached from
/// startup. The kubelet rotates it, and a controller that read it once would
/// keep presenting an expired assertion until it restarted — the classic way a
/// "short-lived credentials" design ends up with a long-lived one.
pub struct WorkloadIdentityTokenSource {
    /// Path the projected token volume is mounted at.
    token_path: PathBuf,
    /// STS audience, as rendered by the installation (#3109).
    audience: String,
    /// Scope requested for the exchanged token.
    scope: String,
    sts_endpoint: String,
    client: reqwest::Client,
    /// Last exchanged token and when it stops being usable.
    cached: Mutex<Option<(String, Instant)>>,
}

/// Federated tokens are minted for an hour; refreshing a few minutes early
/// avoids handing an about-to-expire token to a request that then takes longer
/// than the remainder.
const TOKEN_REFRESH_MARGIN: Duration = Duration::from_secs(300);

impl WorkloadIdentityTokenSource {
    pub fn new(token_path: impl Into<PathBuf>, audience: impl Into<String>) -> Self {
        Self {
            token_path: token_path.into(),
            audience: audience.into(),
            scope: "https://www.googleapis.com/auth/cloud-platform".to_string(),
            sts_endpoint: "https://sts.googleapis.com/v1/token".to_string(),
            client: reqwest::Client::new(),
            cached: Mutex::new(None),
        }
    }

    /// Point the exchange at another STS endpoint. Tests only — production
    /// reads the default above.
    pub fn with_sts_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.sts_endpoint = endpoint.into();
        self
    }

    /// The form body of the exchange, as a list of pairs.
    ///
    /// Split out so the shape is assertable without a network: the audience and
    /// the subject-token type are the two fields that decide whether STS will
    /// accept a KSA assertion at all, and getting either wrong fails at runtime
    /// with a message that names neither.
    pub fn exchange_form(&self, subject_token: &str) -> Vec<(&'static str, String)> {
        vec![
            (
                "grant_type",
                "urn:ietf:params:oauth:grant-type:token-exchange".to_string(),
            ),
            ("audience", self.audience.clone()),
            ("scope", self.scope.clone()),
            (
                "requested_token_type",
                "urn:ietf:params:oauth:token-type:access_token".to_string(),
            ),
            ("subject_token", subject_token.to_string()),
            (
                "subject_token_type",
                "urn:ietf:params:oauth:token-type:jwt".to_string(),
            ),
        ]
    }

    async fn exchange(&self) -> Result<String, IssuerError> {
        if let Some((token, expires_at)) = self.cached.lock().expect("token cache").clone() {
            if Instant::now() + TOKEN_REFRESH_MARGIN < expires_at {
                return Ok(token);
            }
        }

        let assertion = tokio::fs::read_to_string(&self.token_path)
            .await
            .map_err(|err| {
                // The path, never the contents.
                IssuerError::Upstream(format!(
                    "read projected token at {}: {err}",
                    self.token_path.display()
                ))
            })?;

        let form = self.exchange_form(assertion.trim());
        let response = self
            .client
            .post(&self.sts_endpoint)
            .form(&form)
            .send()
            .await
            .map_err(|err| IssuerError::Upstream(format!("token exchange: {err}")))?;
        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|err| IssuerError::Upstream(format!("token exchange response: {err}")))?;
        if !status.is_success() {
            // Deliberately not the body: a failed exchange commonly echoes the
            // assertion back in its error detail.
            return Err(IssuerError::Upstream(format!(
                "token exchange rejected with {status}"
            )));
        }
        let token = body["access_token"]
            .as_str()
            .ok_or_else(|| IssuerError::Upstream("token exchange returned no token".into()))?
            .to_string();
        let lifetime = Duration::from_secs(body["expires_in"].as_u64().unwrap_or(3600));
        *self.cached.lock().expect("token cache") =
            Some((token.clone(), Instant::now() + lifetime));
        Ok(token)
    }
}

impl AccessTokenSource for WorkloadIdentityTokenSource {
    fn token<'a>(&'a self) -> BoxFuture<'a, Result<String, IssuerError>> {
        Box::pin(self.exchange())
    }
}

/// Issues from a CA Service pool.
pub struct CasIssuer {
    id: IssuerId,
    pool: CaPool,
    tokens: Box<dyn AccessTokenSource>,
    client: reqwest::Client,
    endpoint: String,
}

impl CasIssuer {
    pub fn new(pool: CaPool, tokens: Box<dyn AccessTokenSource>) -> Self {
        Self {
            id: IssuerId::new(pool.resource()),
            pool,
            tokens,
            client: reqwest::Client::new(),
            endpoint: "https://privateca.googleapis.com/v1".to_string(),
        }
    }

    /// Point at another API endpoint. Tests only.
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    /// `POST .../caPools/N/certificates?certificateId=…`
    pub fn certificates_url(&self, certificate_id: &str) -> String {
        format!(
            "{}/{}/certificates?certificateId={}",
            self.endpoint,
            self.pool.resource(),
            certificate_id
        )
    }

    /// The request body.
    ///
    /// Two fields, and the absences matter more than the presences. No
    /// `config` — that is the requester-supplied certificate description the
    /// pool refuses (`allow_config_based_issuance = false`, #3109). No
    /// `issuingCertificateAuthorityId` — the pool picks, so retiring a CA is a
    /// pool-level operation rather than a redeploy of everything that requests
    /// from it.
    pub fn request_body(request: &IssuanceRequest) -> Value {
        json!({
            "pemCsr": request.csr_pem,
            "lifetime": format!("{}s", request.lifetime.as_secs()),
        })
    }

    /// A certificate id derived from the CSR.
    ///
    /// Deterministic, so a retry after a timeout re-addresses the certificate
    /// the CA may already have issued instead of minting a sibling. Scoped by
    /// instance and purpose so two workloads cannot collide, and truncated to
    /// CA Service's 63-character limit.
    pub fn certificate_id(request: &IssuanceRequest) -> String {
        let fingerprint = hex_sha256(request.csr_pem.as_bytes());
        let id = format!(
            "{}-{}-{}",
            request.scope.instance,
            request.purpose.as_str(),
            &fingerprint[..16]
        );
        id.chars().take(63).collect()
    }

    async fn request(&self, request: IssuanceRequest) -> Result<IssuedMaterial, IssuerError> {
        let token = self.tokens.token().await?;
        let url = self.certificates_url(&Self::certificate_id(&request));
        let response = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(&Self::request_body(&request))
            .send()
            .await
            .map_err(|err| IssuerError::Upstream(format!("certificate request: {err}")))?;
        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|err| IssuerError::Upstream(format!("certificate response: {err}")))?;
        if !status.is_success() {
            let reason = body["error"]["message"].as_str().unwrap_or("no detail");
            return Err(IssuerError::Upstream(format!(
                "certificate request rejected with {status}: {reason}"
            )));
        }
        Self::material(&self.id, &body)
    }

    /// Turn a CA Service response into material.
    ///
    /// Validity and fingerprint are parsed out of the returned certificate, not
    /// read from the response envelope. The certificate is the thing that will
    /// be presented; if the two ever disagreed, believing the envelope would
    /// mean scheduling renewal against a date nothing enforces.
    pub fn material(issuer: &IssuerId, body: &Value) -> Result<IssuedMaterial, IssuerError> {
        let certificate_pem = body["pemCertificate"]
            .as_str()
            .ok_or_else(|| IssuerError::Malformed("response carried no certificate".into()))?
            .to_string();
        let chain_pem = body["pemCertificateChain"]
            .as_array()
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(Value::as_str)
                    .map(|pem| pem.trim_end().to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
        let facts = parse_leaf(&certificate_pem).map_err(IssuerError::Malformed)?;
        Ok(IssuedMaterial {
            issuer: issuer.clone(),
            certificate_pem,
            chain_pem,
            not_before: facts.not_before,
            not_after: facts.not_after,
            fingerprint: facts.fingerprint,
        })
    }

    /// Fetch the pool's anchors.
    async fn anchors(&self) -> Result<String, IssuerError> {
        let token = self.tokens.token().await?;
        let url = format!("{}/{}:fetchCaCerts", self.endpoint, self.pool.resource());
        let response = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(&json!({}))
            .send()
            .await
            .map_err(|err| IssuerError::Upstream(format!("fetch CA certs: {err}")))?;
        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|err| IssuerError::Upstream(format!("fetch CA certs response: {err}")))?;
        if !status.is_success() {
            return Err(IssuerError::Upstream(format!(
                "fetch CA certs rejected with {status}"
            )));
        }
        Ok(Self::anchors_from(&body))
    }

    /// Flatten a `fetchCaCerts` response into concatenated PEM.
    pub fn anchors_from(body: &Value) -> String {
        body["caCerts"]
            .as_array()
            .map(|chains| {
                chains
                    .iter()
                    .filter_map(|chain| chain["certificates"].as_array())
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(|pem| pem.trim_end().to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default()
    }
}

impl Issuer for CasIssuer {
    fn id(&self) -> IssuerId {
        self.id.clone()
    }

    fn issue<'a>(
        &'a self,
        request: IssuanceRequest,
    ) -> BoxFuture<'a, Result<IssuedMaterial, IssuerError>> {
        Box::pin(self.request(request))
    }

    fn trust_anchor_pem<'a>(&'a self) -> BoxFuture<'a, Result<String, IssuerError>> {
        Box::pin(self.anchors())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::profile::{
        CertificateIdentity, CertificateProfile, InstanceScope, Purpose,
    };

    fn scope() -> InstanceScope {
        InstanceScope::new("lumen", "lumen", "lumen-prod.svc.id.goog")
    }

    fn request() -> IssuanceRequest {
        let profile = CertificateProfile::new(
            &scope(),
            Purpose::Peer,
            "lumen-0.lumen-headless.lumen.svc.cluster.local",
            CertificateIdentity {
                dns_names: vec!["lumen-0.lumen-headless.lumen.svc.cluster.local".into()],
                spiffe_uri: Some("spiffe://lumen-prod.svc.id.goog/ns/lumen/sa/lumen".into()),
            },
            Duration::from_secs(3_600),
            Duration::from_secs(900),
            Duration::from_secs(0),
        )
        .unwrap();
        IssuanceRequest::build(&scope(), &profile).unwrap().0
    }

    fn pool() -> CaPool {
        CaPool {
            project: "axiom-prod".into(),
            location: "us-central1".into(),
            pool: "lumen-issuing".into(),
        }
    }

    struct StaticToken;
    impl AccessTokenSource for StaticToken {
        fn token<'a>(&'a self) -> BoxFuture<'a, Result<String, IssuerError>> {
            Box::pin(futures::future::ready(Ok("test-token".to_string())))
        }
    }

    #[test]
    fn a_pool_resource_name_round_trips() {
        let resource = "projects/axiom-prod/locations/us-central1/caPools/lumen-issuing";
        assert_eq!(CaPool::parse(resource).unwrap(), pool());
        assert_eq!(pool().resource(), resource);
    }

    #[test]
    fn something_that_is_not_a_pool_is_rejected_rather_than_half_parsed() {
        for bad in [
            "projects/axiom-prod/locations/us-central1",
            "projects//locations/us-central1/caPools/lumen-issuing",
            "caPools/lumen-issuing",
            "",
        ] {
            assert!(CaPool::parse(bad).is_err(), "accepted {bad:?}");
        }
    }

    #[test]
    fn the_request_body_asks_for_nothing_the_pool_would_refuse() {
        let body = CasIssuer::request_body(&request());
        let fields = body.as_object().unwrap();
        assert!(fields.contains_key("pemCsr"));
        assert_eq!(body["lifetime"], json!("3600s"));
        assert!(
            !fields.contains_key("config"),
            "the pool refuses config-based issuance (#3109); asking anyway turns a policy \
             into a runtime error"
        );
        assert!(!fields.contains_key("issuingCertificateAuthorityId"));
    }

    #[test]
    fn the_same_csr_addresses_the_same_certificate() {
        let request = request();
        let first = CasIssuer::certificate_id(&request);
        let second = CasIssuer::certificate_id(&request);
        assert_eq!(
            first, second,
            "a retry after a timeout must re-address the certificate the CA may already have \
             issued, not mint a sibling"
        );
        assert!(first.starts_with("lumen-peer-"));
        assert!(first.len() <= 63);
    }

    #[test]
    fn different_requests_do_not_collide() {
        assert_ne!(
            CasIssuer::certificate_id(&request()),
            CasIssuer::certificate_id(&request()),
            "each request carries a fresh key, so each addresses its own certificate"
        );
    }

    #[test]
    fn the_url_names_the_pool_and_the_certificate() {
        let issuer = CasIssuer::new(pool(), Box::new(StaticToken));
        let url = issuer.certificates_url("lumen-peer-0123456789abcdef");
        assert_eq!(
            url,
            "https://privateca.googleapis.com/v1/projects/axiom-prod/locations/us-central1\
             /caPools/lumen-issuing/certificates?certificateId=lumen-peer-0123456789abcdef"
        );
    }

    #[test]
    fn the_issuer_id_is_the_pool_it_issues_from() {
        let issuer = CasIssuer::new(pool(), Box::new(StaticToken));
        assert_eq!(issuer.id(), IssuerId::new(pool().resource()));
    }

    #[test]
    fn the_exchange_presents_a_ksa_assertion_and_nothing_else() {
        let source = WorkloadIdentityTokenSource::new(
            "/var/run/secrets/tokens/gcp-ksa/token",
            "//iam.googleapis.com/projects/1/locations/global/workloadIdentityPools/p/providers/v",
        );
        let form: std::collections::BTreeMap<&str, String> =
            source.exchange_form("assertion").into_iter().collect();
        assert_eq!(
            form["grant_type"],
            "urn:ietf:params:oauth:grant-type:token-exchange"
        );
        assert_eq!(form["subject_token_type"], "urn:ietf:params:oauth:token-type:jwt");
        assert_eq!(form["subject_token"], "assertion");
        assert!(
            !form.contains_key("client_secret") && !form.contains_key("assertion_type"),
            "there is no long-lived credential in this exchange; a field for one is a field \
             somebody will eventually fill in"
        );
    }

    #[test]
    fn a_response_is_believed_only_as_far_as_the_certificate_in_it() {
        // A response whose envelope is missing entirely still yields validity,
        // because validity is read from the leaf.
        let issuer = EphemeralHelper::material();
        let body = json!({
            "pemCertificate": issuer.0,
            "pemCertificateChain": [issuer.1],
        });
        let material = CasIssuer::material(&IssuerId::new("pool"), &body).unwrap();
        assert_eq!(material.not_after, issuer.2);
    }

    #[test]
    fn a_response_without_a_certificate_is_an_error_not_an_empty_leaf() {
        let body = json!({ "pemCertificateChain": [] });
        assert!(CasIssuer::material(&IssuerId::new("pool"), &body).is_err());
    }

    #[test]
    fn anchors_flatten_across_every_chain_the_pool_reports() {
        let body = json!({
            "caCerts": [
                { "certificates": ["-----BEGIN CERTIFICATE-----\nQQ==\n-----END CERTIFICATE-----"] },
                { "certificates": ["-----BEGIN CERTIFICATE-----\nQg==\n-----END CERTIFICATE-----"] },
            ]
        });
        let anchors = CasIssuer::anchors_from(&body);
        assert_eq!(anchors.matches("BEGIN CERTIFICATE").count(), 2);
    }

    /// Signs one leaf with the in-process CA so the response-parsing tests have
    /// a real certificate to parse, without reaching a network.
    struct EphemeralHelper;
    impl EphemeralHelper {
        fn material() -> (String, String, chrono::DateTime<chrono::Utc>) {
            use crate::certificate::ephemeral::{instant, EphemeralIssuer};
            let issuer = EphemeralIssuer::new("pool", instant(2026, 7, 1, 12));
            let material = futures::executor::block_on(issuer.issue(request())).unwrap();
            (
                material.certificate_pem.clone(),
                material.chain_pem.clone(),
                material.not_after,
            )
        }
    }
}
// HANDWRITE-END
