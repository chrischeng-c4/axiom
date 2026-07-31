// HANDWRITE-BEGIN gap="missing-generator:logic:lumen-operator-issuer-config" tracker="#3222" reason="Always-compiled operator issuer configuration grammar shared by offline render and runtime operator."
//! Operator certificate issuer configuration types, validation, and errors.
//!
//! Compiled unconditionally so `lumen k8s operator render` can validate issuer
//! flags in offline builds without pulling in kube-rs or service-k8s (#3222).

/// Operator certificate issuer mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssuerMode {
    Ephemeral,
    Cas,
}

/// Errors occurring during operator issuer configuration validation or resolution.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum IssuerConfigError {
    #[error("missing issuer mode: expected 'cas' or 'ephemeral'")]
    MissingMode,
    #[error("unrecognized issuer mode '{0}': expected 'cas' or 'ephemeral'")]
    UnrecognizedMode(String),
    #[error("trust_domain is required for issuer mode '{0}'")]
    MissingTrustDomain(String),
    #[error("invalid trust_domain: {0}")]
    InvalidTrustDomain(String),
    #[error("CAS-only field '{0}' is forbidden in ephemeral mode")]
    ForbiddenCasField(String),
    #[error("ca_pool is required in cas mode")]
    MissingCaPool,
    #[error("invalid ca_pool resource: {0}")]
    MalformedCaPool(String),
}

/// Unvalidated raw configuration inputs for the operator issuer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RawIssuerConfig {
    pub mode: Option<String>,
    pub trust_domain: Option<String>,
    pub ca_pool: Option<String>,
}

/// Validated operator issuer configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorIssuerConfig {
    pub mode: IssuerMode,
    pub trust_domain: String,
    pub ca_pool_resource: Option<String>,
}

#[cfg(feature = "operator")]
impl OperatorIssuerConfig {
    /// Parsed [`service_k8s::certificate::cas::CaPool`] resource when feature `operator` is enabled.
    pub fn ca_pool(&self) -> Option<service_k8s::certificate::cas::CaPool> {
        self.ca_pool_resource
            .as_ref()
            .and_then(|r| service_k8s::certificate::cas::CaPool::parse(r).ok())
    }

    /// Issuer-resolution seam: constructs the concrete [`service_k8s::certificate::Issuer`].
    pub fn resolve_issuer(
        &self,
    ) -> Result<Box<dyn service_k8s::certificate::Issuer>, IssuerConfigError> {
        match self.mode {
            IssuerMode::Ephemeral => {
                let issuer = service_k8s::certificate::EphemeralIssuer::new(
                    "ephemeral-operator-issuer",
                    chrono::Utc::now(),
                );
                Ok(Box::new(issuer))
            }
            IssuerMode::Cas => {
                let pool_str = self
                    .ca_pool_resource
                    .as_ref()
                    .ok_or(IssuerConfigError::MissingCaPool)?;
                let pool = service_k8s::certificate::cas::CaPool::parse(pool_str)
                    .map_err(|err| IssuerConfigError::MalformedCaPool(err.to_string()))?;
                let token_source = service_k8s::certificate::GkeMetadataTokenSource::new();
                let issuer = service_k8s::certificate::CasIssuer::new(pool, Box::new(token_source));
                Ok(Box::new(issuer))
            }
        }
    }
}

fn validate_trust_domain(td: Option<String>, mode: &str) -> Result<String, IssuerConfigError> {
    let raw = match td {
        Some(ref s) if !s.trim().is_empty() => s.trim(),
        _ => return Err(IssuerConfigError::MissingTrustDomain(mode.to_string())),
    };
    if raw.contains('\n')
        || raw.contains('\r')
        || raw.contains('"')
        || raw.contains('\'')
        || raw.contains('\\')
        || raw.contains(' ')
    {
        return Err(IssuerConfigError::InvalidTrustDomain(format!(
            "contains invalid characters or newline: '{raw}'"
        )));
    }
    if raw.starts_with('-') || raw.ends_with('-') || raw.starts_with('.') || raw.ends_with('.') {
        return Err(IssuerConfigError::InvalidTrustDomain(format!(
            "cannot start or end with '-' or '.': '{raw}'"
        )));
    }
    for c in raw.chars() {
        if !matches!(c, 'a'..='z' | '0'..='9' | '.' | '-') {
            return Err(IssuerConfigError::InvalidTrustDomain(format!(
                "must be DNS-shaped lowercase alphanumeric, '.', or '-': '{raw}'"
            )));
        }
    }
    Ok(raw.to_string())
}

impl RawIssuerConfig {
    /// Validate raw inputs into a closed, typed [`OperatorIssuerConfig`].
    pub fn validate(self) -> Result<OperatorIssuerConfig, IssuerConfigError> {
        let mode_str = match self.mode {
            Some(ref s) if !s.trim().is_empty() => s.trim().to_lowercase(),
            _ => return Err(IssuerConfigError::MissingMode),
        };

        match mode_str.as_str() {
            "ephemeral" => {
                let trust_domain = validate_trust_domain(self.trust_domain, "ephemeral")?;

                if let Some(ref pool) = self.ca_pool {
                    if !pool.trim().is_empty() {
                        return Err(IssuerConfigError::ForbiddenCasField("ca_pool".into()));
                    }
                }

                Ok(OperatorIssuerConfig {
                    mode: IssuerMode::Ephemeral,
                    trust_domain,
                    ca_pool_resource: None,
                })
            }
            "cas" => {
                let trust_domain = validate_trust_domain(self.trust_domain, "cas")?;

                let pool_str = match self.ca_pool {
                    Some(ref p) if !p.trim().is_empty() => p.trim(),
                    _ => return Err(IssuerConfigError::MissingCaPool),
                };

                #[cfg(feature = "operator")]
                {
                    service_k8s::certificate::cas::CaPool::parse(pool_str)
                        .map_err(|err| IssuerConfigError::MalformedCaPool(err.to_string()))?;
                }
                #[cfg(not(feature = "operator"))]
                {
                    let parts: Vec<&str> = pool_str.split('/').collect();
                    if parts.len() != 6
                        || parts[0] != "projects"
                        || parts[2] != "locations"
                        || parts[4] != "caPools"
                        || parts[1].is_empty()
                        || parts[3].is_empty()
                        || parts[5].is_empty()
                    {
                        return Err(IssuerConfigError::MalformedCaPool(format!(
                            "expected projects/P/locations/L/caPools/N, got {pool_str}"
                        )));
                    }
                    for part in [parts[1], parts[3], parts[5]] {
                        if part.chars().any(|c| {
                            c.is_control() || matches!(c, ' ' | '"' | '\'' | '\\' | '\n' | '\r')
                        }) {
                            return Err(IssuerConfigError::MalformedCaPool(format!(
                                "unsafe character in ca_pool: {pool_str}"
                            )));
                        }
                        if !part
                            .chars()
                            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_'))
                        {
                            return Err(IssuerConfigError::MalformedCaPool(format!(
                                "invalid segment in ca_pool: {pool_str}"
                            )));
                        }
                    }
                }

                Ok(OperatorIssuerConfig {
                    mode: IssuerMode::Cas,
                    trust_domain,
                    ca_pool_resource: Some(pool_str.to_string()),
                })
            }
            _ => Err(IssuerConfigError::UnrecognizedMode(mode_str)),
        }
    }
}
// HANDWRITE-END
