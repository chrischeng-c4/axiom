// SPEC-MANAGED: libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `<cli> connect` — the k8s-native service CLI's port-forward lifecycle +
//! token-registry Secret resolution (feature `k8s`). Extracted from `lumen
//! connect` (#1321/#1376): every k8s-native service CLI wants the same
//! `kubectl port-forward` process lifecycle and the same token-registry
//! Secret convention (map key IS the bearer token), so this module owns the
//! reusable primitives. Each adopter supplies only its own flag surface,
//! CR-kind lookup convention, and role mapping into [`Role`] — see
//! `projects/lumen/src/bin/lumen.rs`'s `connect`/`resolve_token` for the
//! reference thin adapter.

use std::collections::HashMap;
use std::process::{Child, Command};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

/// The `token-registry.json` key every token-registry Secret stores its
/// payload under (see `lumen llm --topic auth`'s Secret shape).
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
pub const TOKEN_REGISTRY_SECRET_KEY: &str = "token-registry.json";

/// RAII child-process guard: kills + reaps on drop so a spawned `kubectl
/// port-forward` never survives its wrapped command. Prior art:
/// `projects/preview/tests/kind_lifecycle.rs`'s `ChildGuard`, generalized
/// here over any `std::process::Command` so it is unit-testable with a fake
/// child instead of requiring a real cluster.
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
pub struct ChildGuard {
    child: Child,
}

/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
impl ChildGuard {
    pub fn spawn(command: &mut Command) -> Result<Self> {
        let child = command.spawn().context("spawn child process")?;
        Ok(Self { child })
    }
}

/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Bind an ephemeral local port and immediately release it, returning the
/// number `kubectl port-forward` should target. There is an inherent
/// TOCTOU race (someone else could bind it first), the same tradeoff
/// `projects/preview/tests/kind_lifecycle.rs::free_local_port` makes.
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
pub fn free_local_port() -> Result<u16> {
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", 0)).context("bind ephemeral local port")?;
    Ok(listener.local_addr().context("read local addr")?.port())
}

/// Poll `127.0.0.1:port` until a TCP connect succeeds or `timeout` elapses —
/// the port-forward readiness gate: no fixed sleep, no dependency on
/// kubectl's own stdout.
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
pub fn wait_for_local_port_ready(port: u16, timeout: Duration) -> Result<()> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            anyhow::bail!("port-forward to 127.0.0.1:{port} never became ready within {timeout:?}");
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Role hierarchy for token-registry Secret coverage checks: `Admin` ⊇
/// `Write` ⊇ `Read`. Structurally mirrors `service_auth::Role`; kept
/// independent here (rather than depending on `service-auth`) because
/// `service-auth` itself depends on `cli-std` — depending back would be a
/// dependency cycle. Adopters that already carry `service_auth::Role`
/// convert between the two at the call site (their own "role mapping", per
/// the `connect` adapter convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
pub enum Role {
    Read,
    Write,
    Admin,
}

/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
impl Role {
    /// Whether this role meets or exceeds `needed`.
    pub fn covers(self, needed: Role) -> bool {
        self >= needed
    }
}

/// A bearer token's resolved claims, as stored in a token-registry Secret:
/// who (`subject`) and what they may do, keyed by a generic resource string
/// (a service's collection/namespace/etc). The literal key `*` is a
/// wildcard grant applied when no more specific entry matches.
#[derive(Debug, Clone, Deserialize)]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
pub struct TokenClaims {
    pub subject: String,
    /// `resource` → `Role`. The literal key `*` is a wildcard.
    #[serde(default)]
    pub roles: HashMap<String, Role>,
}

/// Pure: extract `spec.tokensSecret` from a CR's `kubectl get -o json`
/// output (the shared token-registry-Secret CR convention).
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
pub fn cr_tokens_secret(cr_json: &serde_json::Value) -> Option<String> {
    cr_json["spec"]["tokensSecret"].as_str().map(str::to_string)
}

/// Run `kubectl get <resource> <name> -n <namespace> -o json` (optionally
/// through `--context`) and parse the result.
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
pub fn kubectl_get_json(
    context: Option<&str>,
    resource: &str,
    name: &str,
    namespace: &str,
) -> Result<serde_json::Value> {
    let mut cmd = Command::new("kubectl");
    if let Some(ctx) = context {
        cmd.args(["--context", ctx]);
    }
    cmd.args(["get", resource, name, "-n", namespace, "-o", "json"]);
    let output = cmd
        .output()
        .with_context(|| format!("run kubectl get {resource} {name} -n {namespace}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "kubectl get {resource} {name} -n {namespace} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("parse kubectl get {resource} {name} JSON"))
}

/// Resolve a CR's `spec.tokensSecret` (`None` when unset). `resource_kind`
/// is the CRD's kubectl resource name (e.g. `"lumen"`) — the CR-kind lookup
/// convention stays each adopter's own.
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
pub fn resolve_cr_tokens_secret(
    context: Option<&str>,
    namespace: &str,
    resource_kind: &str,
    cr: &str,
) -> Result<Option<String>> {
    let cr_json = kubectl_get_json(context, resource_kind, cr, namespace)?;
    Ok(cr_tokens_secret(&cr_json))
}

/// Pure: decode a Kubernetes Secret's `data.<key>` (base64) field into raw
/// bytes. `kubectl get secret -o json` always base64-encodes `.data`.
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
pub fn secret_data_bytes(secret_json: &serde_json::Value, key: &str) -> Result<Vec<u8>> {
    use base64::Engine;
    let encoded = secret_json["data"][key]
        .as_str()
        .with_context(|| format!("secret has no data key `{key}`"))?;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .context("base64-decode secret data")
}

/// Pick the first registry token whose roles cover `role` for `collection`
/// (falling back to the wildcard `*` grant). Pure — unit-testable without
/// any I/O; deterministic tie-break is not needed since callers name a
/// specific role/collection scope for their own token.
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
pub fn select_token(
    registry: &HashMap<String, TokenClaims>,
    role: Role,
    collection: Option<&str>,
) -> Option<String> {
    registry.iter().find_map(|(token, claims)| {
        let granted = collection
            .and_then(|c| claims.roles.get(c))
            .or_else(|| claims.roles.get("*"));
        granted
            .is_some_and(|granted| granted.covers(role))
            .then(|| token.clone())
    })
}

/// Resolve a usable bearer token without the caller decoding the
/// Secret/JSON by hand. Precedence: `explicit_token` (an already-resolved
/// flag/env value) wins; otherwise, when `namespace`/`secret` are both set,
/// fetch the Secret via kubectl, decode its `token-registry.json` key (the
/// same schema `lumen llm --topic auth` documents), and pick a token whose
/// role covers `role` for `collection` (or `*`). Returns `None` when no
/// token can be resolved (e.g. auth-disabled deployments).
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md#source
pub fn resolve_token(
    explicit_token: Option<&str>,
    context: Option<&str>,
    namespace: Option<&str>,
    secret: Option<&str>,
    role: Role,
    collection: Option<&str>,
) -> Result<Option<String>> {
    if let Some(token) = explicit_token {
        return Ok(Some(token.to_string()));
    }
    let (Some(namespace), Some(secret)) = (namespace, secret) else {
        return Ok(None);
    };
    let secret_json = kubectl_get_json(context, "secret", secret, namespace)?;
    let bytes = secret_data_bytes(&secret_json, TOKEN_REGISTRY_SECRET_KEY)?;
    let registry: HashMap<String, TokenClaims> =
        serde_json::from_slice(&bytes).context("parse token-registry.json")?;
    Ok(select_token(&registry, role, collection))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_covers_hierarchy() {
        assert!(Role::Admin.covers(Role::Read));
        assert!(Role::Admin.covers(Role::Admin));
        assert!(!Role::Read.covers(Role::Admin));
        assert!(Role::Write.covers(Role::Read));
    }

    #[test]
    fn select_token_picks_token_covering_role_for_collection_or_wildcard() {
        let mut registry = HashMap::new();
        registry.insert(
            "reader-token".to_string(),
            TokenClaims {
                subject: "reader".into(),
                roles: [("products".to_string(), Role::Read)].into_iter().collect(),
            },
        );
        registry.insert(
            "admin-token".to_string(),
            TokenClaims {
                subject: "admin".into(),
                roles: [("*".to_string(), Role::Admin)].into_iter().collect(),
            },
        );

        let picked = select_token(&registry, Role::Read, Some("products"));
        assert!(matches!(
            picked.as_deref(),
            Some("reader-token") | Some("admin-token")
        ));
        assert_eq!(
            select_token(&registry, Role::Admin, Some("products")).as_deref(),
            Some("admin-token"),
            "only the wildcard admin token covers admin on `products`"
        );

        let mut narrow = HashMap::new();
        narrow.insert(
            "scoped-token".to_string(),
            TokenClaims {
                subject: "scoped".into(),
                roles: [("orders".to_string(), Role::Write)].into_iter().collect(),
            },
        );
        assert!(select_token(&narrow, Role::Read, Some("products")).is_none());
    }

    #[test]
    fn cr_tokens_secret_reads_spec_field() {
        let cr = serde_json::json!({ "spec": { "tokensSecret": "svc-tokens" } });
        assert_eq!(cr_tokens_secret(&cr).as_deref(), Some("svc-tokens"));

        let cr_missing = serde_json::json!({ "spec": {} });
        assert_eq!(cr_tokens_secret(&cr_missing), None);
    }

    #[test]
    fn secret_data_bytes_decodes_base64_field() {
        use base64::Engine;
        let encoded =
            base64::engine::general_purpose::STANDARD.encode(b"{\"tok\":{\"subject\":\"s\"}}");
        let secret = serde_json::json!({ "data": { TOKEN_REGISTRY_SECRET_KEY: encoded } });
        let bytes = secret_data_bytes(&secret, TOKEN_REGISTRY_SECRET_KEY).unwrap();
        assert_eq!(bytes, b"{\"tok\":{\"subject\":\"s\"}}");

        let missing = serde_json::json!({ "data": {} });
        assert!(secret_data_bytes(&missing, TOKEN_REGISTRY_SECRET_KEY).is_err());
    }

    #[test]
    fn wait_for_local_port_ready_succeeds_against_bound_listener() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(wait_for_local_port_ready(port, Duration::from_secs(2)).is_ok());
        drop(listener);
    }

    #[test]
    fn wait_for_local_port_ready_times_out_against_closed_port() {
        let port = free_local_port().unwrap();
        assert!(wait_for_local_port_ready(port, Duration::from_millis(300)).is_err());
    }

    /// The process-management primitive, unit-tested with a real (but
    /// harmless) child process instead of a live cluster's `kubectl
    /// port-forward`.
    #[test]
    fn child_guard_kills_process_on_drop() {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "sleep 5"]);
        let guard = ChildGuard::spawn(&mut cmd).expect("spawn sleep");
        let pid = guard.child.id();
        drop(guard);
        std::thread::sleep(Duration::from_millis(200));
        let status = Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .expect("run kill -0");
        assert!(
            !status.success(),
            "process {pid} should be dead after ChildGuard drop"
        );
    }

    #[test]
    fn child_guard_spawn_nonexistent_binary_errs() {
        let mut cmd = Command::new("cli-std-connect-test-nonexistent-binary-xyz-1376");
        assert!(ChildGuard::spawn(&mut cmd).is_err());
    }

    #[test]
    fn resolve_token_prefers_explicit_token() {
        let resolved = resolve_token(Some("explicit"), None, None, None, Role::Read, None).unwrap();
        assert_eq!(resolved.as_deref(), Some("explicit"));
    }

    #[test]
    fn resolve_token_returns_none_without_namespace_or_secret() {
        let resolved = resolve_token(None, None, None, None, Role::Read, None).unwrap();
        assert_eq!(resolved, None);
    }
}
// CODEGEN-END
