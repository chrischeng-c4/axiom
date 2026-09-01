//! Sift route policy for the shared reverse-proxy runtime.

use std::time::Duration;

use anyhow::{bail, Context, Result};
use axum::{
    http::{Method, Uri},
    Router,
};
use url::Url;

const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone)]
struct SiftRolePolicy {
    store: Url,
    query: Option<Url>,
    max_body_bytes: usize,
}

impl service_http::ReverseProxyPolicy for SiftRolePolicy {
    fn select_upstream(
        &self,
        _method: &Method,
        uri: &Uri,
    ) -> Result<Url, service_http::ReverseProxySelectionError> {
        Ok(if self.query.is_some() && is_query_path(uri.path()) {
            self.query.as_ref().expect("query endpoint checked").clone()
        } else {
            self.store.clone()
        })
    }

    fn max_body_bytes(&self) -> usize {
        self.max_body_bytes
    }

    fn upstream_timeout(&self) -> Duration {
        UPSTREAM_TIMEOUT
    }
}

pub fn gateway_router(
    store_endpoint: &str,
    query_endpoint: &str,
    max_body_bytes: usize,
) -> Result<Router> {
    router(store_endpoint, Some(query_endpoint), max_body_bytes)
}

pub fn query_router(store_endpoint: &str, max_body_bytes: usize) -> Result<Router> {
    router(store_endpoint, None, max_body_bytes)
}

fn router(
    store_endpoint: &str,
    query_endpoint: Option<&str>,
    max_body_bytes: usize,
) -> Result<Router> {
    let store = endpoint(store_endpoint, "store")?;
    let query = query_endpoint
        .map(|value| endpoint(value, "query"))
        .transpose()?;
    service_http::reverse_proxy_router(SiftRolePolicy {
        store,
        query,
        max_body_bytes,
    })
}

fn endpoint(value: &str, role: &str) -> Result<Url> {
    let mut url = Url::parse(value).with_context(|| format!("parse Sift {role} endpoint"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        bail!("Sift {role} endpoint must use http or https and include a host");
    }
    if url.query().is_some() || url.fragment().is_some() {
        bail!("Sift {role} endpoint must not include a query or fragment");
    }
    url.set_path("");
    Ok(url)
}

fn is_query_path(path: &str) -> bool {
    matches!(
        path,
        "/api/v1/query"
            | "/api/v1/logs/tail"
            | "/api/v1/correlate"
            | "/api/v1/services"
            | "/prometheus/api/v1/query"
            | "/prometheus/api/v1/query_range"
    ) || path.starts_with("/api/v1/traces/")
        || path.starts_with("/api/v1/queries/")
}
