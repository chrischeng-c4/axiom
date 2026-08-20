// HANDWRITE-BEGIN gap="missing-generator:e2e-test:lumen-control-plane-projected-identity" tracker="#2877" reason="Drives a real projected-token file through a real router: rotation mid-run, the refusals that must happen before a request is sent, and two control-plane callers whose grants are independent. No generator primitive composes a filesystem credential source with an HTTP authorization matrix."
//! The Lumen control plane authenticating as itself (#2877).
//!
//! Two workloads in this repository call a serving fleet's admin API: the
//! operator's reshard driver and the backup CronJob. Neither is a tenant.
//! Neither has a Google identity. Each has a Kubernetes ServiceAccount, and
//! each presents a token the kubelet minted for `lumen.axiom.dev` and rewrites
//! in place before it expires.
//!
//! What that buys is checked here rather than asserted in a comment:
//!
//! - **Rotation reaches the wire.** The kubelet replaces the file at ~80% of
//!   the token's life, with no restart and no signal. A client that read it
//!   once at startup would authenticate for eight minutes and then fail for as
//!   long as the process lived.
//! - **The refusals happen locally.** A missing mount, an expired token, or one
//!   minted for the API server's own audience is caught before a request is
//!   sent — with an error naming the path, never the material.
//! - **The two callers are independent.** Removing one RoleBinding denies one
//!   caller. If both rode a shared credential, revoking either would revoke
//!   both, and the serving side's access log could not say which one called.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum_test::TestServer;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;

use lumen::api::{router, AppState};
use lumen::auth::{AuthConfig, LumenVerifier, ADMIN_RESOURCE, AUDIENCE};
use lumen::storage::Engine;
use service_auth::k8s::{
    AccessReviewOutcome, CachePolicy, DelegatedAuthConfig, DelegatedAuthenticator,
    ProjectedTokenFile, ResourceAttributes, ReviewBackend, ReviewError, ReviewedIdentity,
    TokenReviewOutcome,
};

const SERVING_NAMESPACE: &str = "lumen-acceptance";
/// The operator runs in its own namespace, as its own ServiceAccount.
const OPERATOR: &str = "system:serviceaccount:lumen-system:lumen-operator";
/// The backup CronJob runs beside the fleet it backs up, as the ServiceAccount
/// the operator renders for it.
const BACKUP: &str = "system:serviceaccount:lumen-acceptance:search-backup";

/// The string that must never appear in any error, log line, or panic message
/// this test can produce. It is carried inside the token's own claims, so a
/// diagnostic that prints "the token" prints this.
const CANARY: &str = "canary-control-plane-token-must-never-be-printed";

// ---------------------------------------------------------------------------
// A scripted apiserver
// ---------------------------------------------------------------------------

/// kube-apiserver, reduced to the two questions Lumen asks it: who is this,
/// and may they do that.
///
/// Tokens map to principals by their `sub` claim, which is what makes the
/// rotation test meaningful — a replacement file carrying a different subject
/// authenticates as a different principal, so "the new token reached the wire"
/// is observable at the serving side rather than inferred from the client.
struct Cluster {
    /// `sub` claim -> the username kube-apiserver would report.
    subjects: HashMap<String, String>,
    /// `(username, resource, name, verb)`, exactly as a SubjectAccessReview
    /// would be asked.
    grants: Mutex<HashSet<(String, String, Option<String>, String)>>,
    /// Every principal that successfully authenticated, in order.
    authenticated: Mutex<Vec<String>>,
}

impl Cluster {
    fn new() -> Self {
        Self {
            subjects: HashMap::from([
                (OPERATOR.to_string(), OPERATOR.to_string()),
                (BACKUP.to_string(), BACKUP.to_string()),
            ]),
            grants: Mutex::new(HashSet::new()),
            authenticated: Mutex::new(Vec::new()),
        }
    }

    fn grant_admin(&self, user: &str) {
        self.grants.lock().expect("grants").insert((
            user.to_string(),
            ADMIN_RESOURCE.to_string(),
            None,
            "delete".to_string(),
        ));
    }

    /// Delete one RoleBinding. The other caller's grant is untouched.
    fn revoke_admin(&self, user: &str) {
        self.grants.lock().expect("grants").remove(&(
            user.to_string(),
            ADMIN_RESOURCE.to_string(),
            None,
            "delete".to_string(),
        ));
    }

    fn authenticated(&self) -> Vec<String> {
        self.authenticated.lock().expect("authenticated").clone()
    }
}

#[async_trait]
impl ReviewBackend for Cluster {
    async fn review_token(
        &self,
        token: &str,
        audiences: &[String],
    ) -> Result<TokenReviewOutcome, ReviewError> {
        // The apiserver validates the signature; this stand-in reads the claims
        // it would have validated. What matters for the test is the mapping
        // from token *bytes* to principal, so a rotated file is a different
        // answer here.
        let Some(subject) = claim(token, "sub") else {
            return Ok(TokenReviewOutcome {
                authenticated: false,
                identity: ReviewedIdentity::default(),
                audiences: Vec::new(),
                error: Some("not a token".into()),
            });
        };
        Ok(match self.subjects.get(&subject) {
            Some(username) => {
                self.authenticated
                    .lock()
                    .expect("authenticated")
                    .push(username.clone());
                TokenReviewOutcome {
                    authenticated: true,
                    identity: ReviewedIdentity {
                        username: username.clone(),
                        ..Default::default()
                    },
                    audiences: audiences.to_vec(),
                    error: None,
                }
            }
            None => TokenReviewOutcome {
                authenticated: false,
                identity: ReviewedIdentity::default(),
                audiences: Vec::new(),
                error: Some("unknown subject".into()),
            },
        })
    }

    async fn review_access(
        &self,
        identity: &ReviewedIdentity,
        attributes: &ResourceAttributes,
    ) -> Result<AccessReviewOutcome, ReviewError> {
        let key = (
            identity.username.clone(),
            attributes.resource.clone(),
            attributes.name.clone(),
            attributes.verb.clone(),
        );
        Ok(if self.grants.lock().expect("grants").contains(&key) {
            AccessReviewOutcome::allow()
        } else {
            AccessReviewOutcome::deny("no RoleBinding grants this")
        })
    }
}

/// Read one claim out of a token without verifying anything — the scripted
/// apiserver's stand-in for a signature check it has no key for.
fn claim(token: &str, name: &str) -> Option<String> {
    let payload = token.split('.').nth(1)?;
    let bytes = base64_url_decode(payload)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    value[name].as_str().map(str::to_string)
}

fn base64_url_decode(input: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits = 0u32;
    for byte in input.bytes() {
        let index = ALPHABET.iter().position(|candidate| *candidate == byte)? as u32;
        buffer = (buffer << 6) | index;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
        }
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// Projected token fixtures
// ---------------------------------------------------------------------------

/// A projected-token mount, in a temp directory the test owns.
///
/// [`Mount::rotate`] is the kubelet: it replaces the file's contents in place
/// while whoever holds the path keeps holding the same path.
struct Mount {
    dir: PathBuf,
}

impl Mount {
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "lumen-control-plane-{label}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("create mount");
        Self { dir }
    }

    fn path(&self) -> PathBuf {
        self.dir.join("token")
    }

    fn write(&self, token: &str) {
        std::fs::write(self.path(), token).expect("write projected token");
    }

    fn rotate(&self, token: &str) {
        self.write(token);
    }

    fn source(&self) -> ProjectedTokenFile {
        ProjectedTokenFile::new(self.path(), AUDIENCE)
    }
}

impl Drop for Mount {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Mint a token the way the apiserver would: an audience, an expiry, a
/// subject — plus the canary, so anything that echoes this token is caught.
fn mint(subject: &str, audience: &str, expires_in_secs: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_secs() as i64;
    let claims = json!({
        "sub": subject,
        "aud": [audience],
        "exp": now + expires_in_secs,
        "iat": now,
        "jti": CANARY,
    });
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"throwaway - nothing here verifies a signature"),
    )
    .expect("mint token")
}

fn serving_fleet(cluster: Arc<Cluster>) -> TestServer {
    let engine = Arc::new(Engine::new());
    // Every review goes to the scripted apiserver. The production cache exists
    // and its revocation bound is #2869's contract, tested there against an
    // injectable clock; what this file is about is *which caller* a binding
    // covers, and a five-minute allow TTL would only make that answer arrive
    // five minutes late.
    let config = DelegatedAuthConfig::new(vec![AUDIENCE.to_string()])
        .expect("audience")
        .with_cache_policy(CachePolicy {
            allow_ttl: Duration::ZERO,
            deny_ttl: Duration::ZERO,
            stale_window: Duration::ZERO,
            ..CachePolicy::default()
        });
    let verifier = Arc::new(LumenVerifier::with_authenticator(
        SERVING_NAMESPACE,
        Arc::new(DelegatedAuthenticator::new(cluster, config)),
    ));
    let state = AppState::new(engine, Arc::new(AuthConfig::required_in(SERVING_NAMESPACE)))
        .with_verifier(verifier);
    TestServer::new(router(state)).expect("server")
}

/// One admin call, carrying whatever the token source produced.
async fn admin_backup(server: &TestServer, source: &ProjectedTokenFile) -> u16 {
    let token = source.read().expect("projected token reads");
    server
        .get("/admin/backup")
        .add_header("authorization", format!("Bearer {}", token.expose()))
        .await
        .status_code()
        .as_u16()
}

// ---------------------------------------------------------------------------
// AC3 — rotation
// ---------------------------------------------------------------------------

/// #2877 AC3: the second call uses the replacement token, in the same process.
///
/// This is the failure mode the whole design is arranged around. Projected
/// tokens expire in ten minutes and the kubelet rewrites the file at around
/// eight. A client that cached the string it read at startup would work
/// perfectly through every test, every deploy, and every manual check — and
/// then start failing at a moment unrelated to any change anyone made.
///
/// The proof is that the serving side sees a different principal, not that the
/// client's string changed: rotation only counts if the new material reaches
/// the wire.
#[tokio::test]
async fn a_rotated_mount_changes_who_the_fleet_sees_without_a_restart() {
    let cluster = Arc::new(Cluster::new());
    cluster.grant_admin(OPERATOR);
    cluster.grant_admin(BACKUP);
    let server = serving_fleet(cluster.clone());

    let mount = Mount::new("rotation");
    mount.write(&mint(OPERATOR, AUDIENCE, 600));
    let source = mount.source();

    assert_eq!(admin_backup(&server, &source).await, 200);

    // The kubelet swaps the file. Nothing restarts, nothing is notified, and
    // the client keeps the same `ProjectedTokenFile` it has held all along.
    mount.rotate(&mint(BACKUP, AUDIENCE, 600));

    assert_eq!(admin_backup(&server, &source).await, 200);
    assert_eq!(
        cluster.authenticated(),
        vec![OPERATOR.to_string(), BACKUP.to_string()],
        "the second call must present the replacement token, not the cached one"
    );
}

// ---------------------------------------------------------------------------
// AC4 — the refusals, and the canary
// ---------------------------------------------------------------------------

/// #2877 AC4/R5: bad credential material stops the action locally, and no
/// diagnostic carries the token.
///
/// Each of these could be left to the serving side, and each would come back as
/// a bare 401 — the same answer a missing RoleBinding gives. Catching them at
/// the mount means the operator reads "no token at
/// /var/run/secrets/lumen.axiom.dev/token" instead of digging through RBAC for
/// a binding that was never the problem.
///
/// The default pod token is the case worth naming: every pod has one, at a
/// path one line away from this one, and the apiserver authenticates it
/// happily. Only the audience distinguishes it.
#[test]
fn bad_material_fails_before_the_request_and_never_prints_itself() {
    let mount = Mount::new("refusals");
    let source = mount.source();

    // Nothing mounted at all — the mount was forgotten, or the volume name in
    // the manifest does not match the one in the pod spec.
    let missing = source.read().expect_err("a missing mount cannot succeed");
    let text = format!("{missing}");
    assert!(
        text.contains("/token"),
        "the error must name the path the operator has to go fix: {text}"
    );

    // A token the apiserver would accept, for the apiserver. This is the
    // default `/var/run/secrets/kubernetes.io/serviceaccount` token, and
    // presenting it to Lumen is the single most likely wiring mistake.
    mount.write(&mint(OPERATOR, "https://kubernetes.default.svc", 600));
    let wrong_audience = source.read().expect_err("a foreign audience cannot succeed");

    // Expired: the pod outlived its token, which is what happens when a client
    // caches one. Ten minutes past, comfortably outside the reader's small
    // clock-skew grace — the callee holds the authoritative clock, so the
    // grace exists to avoid refusing a token the fleet would have accepted.
    mount.write(&mint(OPERATOR, AUDIENCE, -600));
    let expired = source.read().expect_err("an expired token cannot succeed");

    // Not a token at all — a ConfigMap key mounted where a token should be.
    mount.write("not-a-token");
    let malformed = source.read().expect_err("garbage cannot succeed");

    for failure in [&missing, &wrong_audience, &expired, &malformed] {
        for rendering in [format!("{failure}"), format!("{failure:?}")] {
            assert!(
                !rendering.contains(CANARY),
                "a diagnostic leaked token material: {rendering}"
            );
            assert!(
                !rendering.contains("eyJ"),
                "a diagnostic printed the token itself: {rendering}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC5 — two callers, two bindings
// ---------------------------------------------------------------------------

/// #2877 AC5: each control-plane caller stands on its own RoleBinding.
///
/// The point of two ServiceAccounts is that revocation is targeted. Cutting the
/// operator's binding must stop reshard admin calls and leave scheduled backups
/// running — and cutting the backup runner's must do the mirror image. A shared
/// credential would collapse both into one switch, and the serving fleet's
/// access log could not tell an operator-initiated write from a backup read.
#[tokio::test]
async fn revoking_one_binding_denies_one_caller() {
    let cluster = Arc::new(Cluster::new());
    cluster.grant_admin(OPERATOR);
    cluster.grant_admin(BACKUP);
    let server = serving_fleet(cluster.clone());

    let operator_mount = Mount::new("operator");
    operator_mount.write(&mint(OPERATOR, AUDIENCE, 600));
    let operator = operator_mount.source();

    let backup_mount = Mount::new("backup");
    backup_mount.write(&mint(BACKUP, AUDIENCE, 600));
    let backup = backup_mount.source();

    assert_eq!(admin_backup(&server, &operator).await, 200);
    assert_eq!(admin_backup(&server, &backup).await, 200);

    cluster.revoke_admin(OPERATOR);
    assert_eq!(
        admin_backup(&server, &operator).await,
        403,
        "the revoked caller must be denied — and denied, not un-authenticated"
    );
    assert_eq!(
        admin_backup(&server, &backup).await,
        200,
        "the other caller's binding was not touched"
    );

    cluster.grant_admin(OPERATOR);
    cluster.revoke_admin(BACKUP);
    assert_eq!(admin_backup(&server, &operator).await, 200);
    assert_eq!(admin_backup(&server, &backup).await, 403);
}
// HANDWRITE-END
