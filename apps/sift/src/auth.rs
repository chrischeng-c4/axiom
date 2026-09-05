// HANDWRITE-BEGIN gap="sift-shared-bearer-auth" tracker="1604" reason="Adapt shared bearer-token verification to Sift environment configuration and data-plane middleware."
//! Sift authentication. Local deployments may use a static role map. GKE
//! delegates identity and project authorization to the Kubernetes API server.
//!
//! `SIFT_AUTH=required` protects only data-plane routes. Standard health,
//! readiness, metrics, OpenAPI, and docs routes remain operable for platform
//! probes. Production tokens are read from `SIFT_TOKEN_REGISTRY_FILE`; the
//! inline `SIFT_TOKENS` variable is retained for local development.

use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, Context, Result};
use axum::http::{HeaderMap, Method, Uri};
use service_auth::{
    k8s::{
        DelegatedAuthConfig, DelegatedAuthenticator, KubeReviewBackend, ResourceAttributes,
        ReviewBackend,
    },
    AuthError, Role, RoleMapPrincipal, StaticRoleMapVerifier, TokenClaims, Verifier,
};

const TOKEN_REGISTRY_FILE_ENV: &str = "SIFT_TOKEN_REGISTRY_FILE";
const LEGACY_TOKENS_ENV: &str = "SIFT_TOKENS";

#[derive(Debug, Clone)]
pub struct SiftAuthConfig {
    pub required: bool,
    pub tokens: HashMap<String, TokenClaims>,
}

impl SiftAuthConfig {
    pub fn open() -> Self {
        Self {
            required: false,
            tokens: HashMap::new(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let required = match std::env::var("SIFT_AUTH") {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "required" => true,
                "off" | "disabled" => false,
                other => bail!("SIFT_AUTH must be `off`, `disabled`, or `required`; got `{other}`"),
            },
            Err(std::env::VarError::NotPresent) => false,
            Err(error) => bail!("SIFT_AUTH must be valid UTF-8: {error}"),
        };
        let tokens = service_auth::load_registry(
            required,
            TOKEN_REGISTRY_FILE_ENV,
            std::env::var(TOKEN_REGISTRY_FILE_ENV).ok().as_deref(),
            LEGACY_TOKENS_ENV,
            std::env::var(LEGACY_TOKENS_ENV).ok().as_deref(),
        )?;
        Ok(Self { required, tokens })
    }
}

#[derive(Clone)]
enum SiftVerifierInner {
    Static(StaticRoleMapVerifier),
    Kubernetes {
        authenticator: Arc<DelegatedAuthenticator>,
        namespace: Arc<str>,
    },
}

#[derive(Clone)]
pub struct SiftVerifier(SiftVerifierInner);

impl SiftVerifier {
    pub fn new(config: SiftAuthConfig) -> Self {
        Self(SiftVerifierInner::Static(StaticRoleMapVerifier::new(
            config.required,
            config.tokens,
        )))
    }

    pub fn kubernetes(
        backend: Arc<dyn ReviewBackend>,
        audience: &str,
        namespace: &str,
    ) -> Result<Self> {
        let audience = audience.trim();
        let namespace = namespace.trim();
        if audience.is_empty() {
            bail!("Sift Kubernetes audience must not be empty");
        }
        if namespace.is_empty() {
            bail!("Sift serving namespace must not be empty");
        }
        let config = DelegatedAuthConfig::new(vec![audience.to_string()])
            .context("build Sift Kubernetes delegated-auth configuration")?;
        Ok(Self(SiftVerifierInner::Kubernetes {
            authenticator: Arc::new(DelegatedAuthenticator::new(backend, config)),
            namespace: Arc::from(namespace),
        }))
    }

    pub async fn from_env() -> Result<Self> {
        if std::env::var("SIFT_AUTH")
            .ok()
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("kubernetes"))
        {
            let audience =
                std::env::var("SIFT_K8S_AUDIENCE").unwrap_or_else(|_| "sift.axiom.dev".to_string());
            let namespace = std::env::var("POD_NAMESPACE")
                .context("POD_NAMESPACE is required when SIFT_AUTH=kubernetes")?;
            let config = DelegatedAuthConfig::new(vec![audience.trim().to_string()])
                .context("build Sift Kubernetes delegated-auth configuration")?;
            let backend = KubeReviewBackend::in_cluster()
                .await
                .map_err(|error| anyhow::anyhow!("initialize Kubernetes reviews: {error}"))?;
            let probe = ResourceAttributes::new(
                "sift.axiom.dev",
                namespace.trim(),
                "projects",
                Some("sift-delegation-probe".to_string()),
                "get",
            );
            backend
                .probe_delegation(config.audiences(), &probe)
                .await
                .map_err(|error| anyhow::anyhow!("probe Kubernetes auth delegation: {error}"))?;
            return Ok(Self(SiftVerifierInner::Kubernetes {
                authenticator: Arc::new(DelegatedAuthenticator::new(Arc::new(backend), config)),
                namespace: Arc::from(namespace.trim()),
            }));
        }
        Ok(Self::new(SiftAuthConfig::from_env()?))
    }

    pub fn required(&self) -> bool {
        match &self.0 {
            SiftVerifierInner::Static(verifier) => verifier.required(),
            SiftVerifierInner::Kubernetes { .. } => true,
        }
    }

    pub fn is_kubernetes(&self) -> bool {
        matches!(&self.0, SiftVerifierInner::Kubernetes { .. })
    }

    pub async fn authenticate_project(
        &self,
        headers: &HeaderMap,
        project: &str,
        role: Role,
    ) -> std::result::Result<RoleMapPrincipal, AuthError> {
        match &self.0 {
            SiftVerifierInner::Static(verifier) => verifier.authenticate(headers),
            SiftVerifierInner::Kubernetes {
                authenticator,
                namespace,
            } => {
                let token =
                    service_auth::bearer_token(headers).ok_or(AuthError::Unauthenticated)?;
                let caller = authenticator
                    .authenticate(token)
                    .await
                    .map_err(AuthError::from)?;
                let attributes = ResourceAttributes::new(
                    "sift.axiom.dev",
                    namespace.as_ref(),
                    "projects",
                    Some(project.to_string()),
                    role_verb(role),
                );
                authenticator
                    .authorize(&caller, &attributes)
                    .await
                    .map_err(AuthError::from)?;
                Ok(RoleMapPrincipal::Token(TokenClaims {
                    subject: caller.username(),
                    roles: HashMap::from([(
                        if role == Role::Admin {
                            "*".to_string()
                        } else {
                            project.to_string()
                        },
                        role,
                    )]),
                }))
            }
        }
    }
}

fn role_verb(role: Role) -> &'static str {
    match role {
        Role::Read => "get",
        Role::Write => "create",
        Role::Admin => "update",
    }
}

fn http_role(method: &Method, path: &str) -> Role {
    if path.starts_with("/admin/") {
        Role::Admin
    } else if matches!(
        path,
        "/v1/logs" | "/v1/metrics" | "/v1/traces" | "/prometheus/api/v1/write"
    ) || (*method != Method::GET
        && !matches!(
            path,
            "/api/v1/query"
                | "/api/v1/logs/tail"
                | "/api/v1/correlate"
                | "/prometheus/api/v1/query"
                | "/prometheus/api/v1/query_range"
        ))
    {
        Role::Write
    } else {
        Role::Read
    }
}

#[async_trait::async_trait]
impl service_auth::ScopedAuthorization for SiftVerifier {
    type Principal = RoleMapPrincipal;

    async fn authorize_scope(
        &self,
        headers: &HeaderMap,
        method: &Method,
        uri: &Uri,
    ) -> std::result::Result<service_auth::ScopedAuthorizationOutcome<Self::Principal>, AuthError>
    {
        if self.is_kubernetes() && uri.path() == "/mcp" {
            // MCP calls are authorized again when each tool calls the normal API
            // with its explicit project. The outer transport owns Host/Origin.
            return Ok(service_auth::ScopedAuthorizationOutcome::Bypass);
        }
        let project = headers
            .get("x-sift-project")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("");
        if self.is_kubernetes() && project.is_empty() {
            return Err(AuthError::Forbidden(
                "x-sift-project is required for Kubernetes project authorization".into(),
            ));
        }
        let role = http_role(method, uri.path());
        let principal = self.authenticate_project(headers, project, role).await?;
        Ok(service_auth::ScopedAuthorizationOutcome::Authorized(
            principal,
        ))
    }
}

pub use service_auth::scoped_authorization_middleware as auth_middleware;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_config_has_no_tokens() {
        let config = SiftAuthConfig::open();
        assert!(!config.required);
        assert!(config.tokens.is_empty());
    }
}

// HANDWRITE-END
