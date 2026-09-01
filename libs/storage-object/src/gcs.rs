use std::time::Duration;

use reqwest::blocking::{Client, RequestBuilder, Response};
use serde_json::Value;

use crate::{
    validate_key, Object, ObjectMeta, ObjectStore, ObjectStoreError, ObjectVersion, PutCondition,
    Result,
};

pub struct GcsObjectStore {
    bucket: String,
    prefix: String,
    endpoint: String,
    anonymous: bool,
    client: Client,
}

impl GcsObjectStore {
    /// Production constructor. Authentication uses an ADC access token or the
    /// GKE metadata server. Service-account key files are not read.
    pub fn new(bucket: impl Into<String>, prefix: impl Into<String>) -> Result<Self> {
        let bucket = bucket.into();
        let prefix = prefix.into();
        let (endpoint, anonymous) = match std::env::var("STORAGE_EMULATOR_HOST") {
            Ok(value) => (normalize_endpoint(value), true),
            Err(std::env::VarError::NotPresent) => {
                ("https://storage.googleapis.com".to_string(), false)
            }
            Err(error) => return Err(unavailable(error)),
        };
        Self::build(bucket, prefix, endpoint, anonymous)
    }

    pub fn anonymous_emulator(
        bucket: impl Into<String>,
        prefix: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Result<Self> {
        Self::build(bucket.into(), prefix.into(), endpoint.into(), true)
    }

    fn build(bucket: String, prefix: String, endpoint: String, anonymous: bool) -> Result<Self> {
        if bucket.trim().is_empty() || bucket.contains('/') {
            return Err(ObjectStoreError::InvalidKey { key: bucket });
        }
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(unavailable)?;
        Ok(Self {
            bucket,
            prefix: prefix.trim_matches('/').to_string(),
            endpoint: normalize_endpoint(endpoint),
            anonymous,
            client,
        })
    }

    fn full_key(&self, key: &str) -> Result<String> {
        let key = validate_key(key)?;
        Ok(if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}/{key}", self.prefix)
        })
    }

    fn relative_key(&self, key: &str) -> Result<String> {
        if self.prefix.is_empty() {
            return Ok(key.to_string());
        }
        key.strip_prefix(&format!("{}/", self.prefix))
            .map(str::to_string)
            .ok_or_else(|| ObjectStoreError::Corrupt {
                message: format!("GCS returned key outside configured prefix: {key}"),
            })
    }

    fn authorize(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        if self.anonymous {
            return Ok(request);
        }
        Ok(request.bearer_auth(access_token(&self.client)?))
    }

    fn parse_meta(&self, value: &Value) -> Result<ObjectMeta> {
        let full_key = value
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| corrupt("GCS object metadata lacks name"))?;
        let key = self.relative_key(full_key)?;
        let size = value
            .get("size")
            .and_then(Value::as_str)
            .ok_or_else(|| corrupt("GCS object metadata lacks size"))?
            .parse::<u64>()
            .map_err(|error| corrupt(error))?;
        let generation = value
            .get("generation")
            .and_then(Value::as_str)
            .ok_or_else(|| corrupt("GCS object metadata lacks generation"))?;
        Ok(ObjectMeta {
            key,
            size,
            content_type: value
                .get("contentType")
                .and_then(Value::as_str)
                .unwrap_or("application/octet-stream")
                .to_string(),
            version: ObjectVersion::new(generation),
            etag: value
                .get("etag")
                .or_else(|| value.get("md5Hash"))
                .and_then(Value::as_str)
                .map(str::to_string),
            updated: value
                .get("updated")
                .and_then(Value::as_str)
                .map(str::to_string),
        })
    }
}

impl ObjectStore for GcsObjectStore {
    fn put(
        &self,
        key: &str,
        bytes: &[u8],
        content_type: &str,
        condition: PutCondition,
    ) -> Result<ObjectMeta> {
        let full_key = self.full_key(key)?;
        let mut url = format!(
            "{}/upload/storage/v1/b/{}/o?uploadType=media&name={}",
            self.endpoint,
            urlencoding::encode(&self.bucket),
            urlencoding::encode(&full_key),
        );
        match condition {
            PutCondition::Any => {}
            PutCondition::IfAbsent => url.push_str("&ifGenerationMatch=0"),
            PutCondition::IfVersion(version) => {
                if !version.as_str().bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(ObjectStoreError::PreconditionFailed {
                        key: key.to_string(),
                    });
                }
                url.push_str("&ifGenerationMatch=");
                url.push_str(version.as_str());
            }
        }
        let response = self
            .authorize(self.client.post(url))?
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .body(bytes.to_vec())
            .send()
            .map_err(unavailable)?;
        let value: Value = successful(response, key)?.json().map_err(corrupt)?;
        self.parse_meta(&value)
    }

    fn get(&self, key: &str) -> Result<Object> {
        let meta = self.head(key)?;
        let full_key = self.full_key(key)?;
        let url = format!(
            "{}/download/storage/v1/b/{}/o/{}?alt=media",
            self.endpoint,
            urlencoding::encode(&self.bucket),
            urlencoding::encode(&full_key),
        );
        let response = self
            .authorize(self.client.get(url))?
            .send()
            .map_err(unavailable)?;
        let bytes = successful(response, key)?
            .bytes()
            .map_err(unavailable)?
            .to_vec();
        Ok(Object { meta, bytes })
    }

    fn head(&self, key: &str) -> Result<ObjectMeta> {
        let full_key = self.full_key(key)?;
        let url = format!(
            "{}/storage/v1/b/{}/o/{}",
            self.endpoint,
            urlencoding::encode(&self.bucket),
            urlencoding::encode(&full_key),
        );
        let response = self
            .authorize(self.client.get(url))?
            .send()
            .map_err(unavailable)?;
        let value: Value = successful(response, key)?.json().map_err(corrupt)?;
        self.parse_meta(&value)
    }

    fn list(&self, prefix: &str) -> Result<Vec<ObjectMeta>> {
        let full_prefix = if prefix.trim_matches('/').is_empty() {
            self.prefix.clone()
        } else {
            self.full_key(prefix.trim_matches('/'))?
        };
        let mut page_token = None::<String>;
        let mut objects = Vec::new();
        loop {
            let mut url = format!(
                "{}/storage/v1/b/{}/o?prefix={}",
                self.endpoint,
                urlencoding::encode(&self.bucket),
                urlencoding::encode(&full_prefix),
            );
            if let Some(token) = &page_token {
                url.push_str("&pageToken=");
                url.push_str(&urlencoding::encode(token));
            }
            let response = self
                .authorize(self.client.get(url))?
                .send()
                .map_err(unavailable)?;
            let value: Value = successful(response, &full_prefix)?
                .json()
                .map_err(corrupt)?;
            for item in value
                .get("items")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                objects.push(self.parse_meta(item)?);
            }
            page_token = value
                .get("nextPageToken")
                .and_then(Value::as_str)
                .filter(|token| !token.is_empty())
                .map(str::to_string);
            if page_token.is_none() {
                break;
            }
        }
        objects.sort_by(|left, right| left.key.cmp(&right.key));
        Ok(objects)
    }

    fn delete(&self, key: &str) -> Result<()> {
        let full_key = self.full_key(key)?;
        let url = format!(
            "{}/storage/v1/b/{}/o/{}",
            self.endpoint,
            urlencoding::encode(&self.bucket),
            urlencoding::encode(&full_key),
        );
        let response = self
            .authorize(self.client.delete(url))?
            .send()
            .map_err(unavailable)?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }
        successful(response, key)?;
        Ok(())
    }
}

fn normalize_endpoint(value: impl Into<String>) -> String {
    let value = value.into();
    let value = if value.starts_with("http://") || value.starts_with("https://") {
        value
    } else {
        format!("http://{value}")
    };
    value.trim_end_matches('/').to_string()
}

fn successful(response: Response, key: &str) -> Result<Response> {
    if response.status().is_success() {
        return Ok(response);
    }
    match response.status() {
        reqwest::StatusCode::NOT_FOUND => Err(ObjectStoreError::NotFound {
            key: key.to_string(),
        }),
        reqwest::StatusCode::PRECONDITION_FAILED | reqwest::StatusCode::CONFLICT => {
            Err(ObjectStoreError::PreconditionFailed {
                key: key.to_string(),
            })
        }
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
            Err(ObjectStoreError::Unauthorized)
        }
        status => {
            let body = response.text().unwrap_or_default();
            Err(ObjectStoreError::Unavailable {
                message: format!("GCS returned {status}: {}", truncate(&body, 1024)),
            })
        }
    }
}

fn access_token(client: &Client) -> Result<String> {
    for name in ["GOOGLE_OAUTH_ACCESS_TOKEN", "GCS_ACCESS_TOKEN"] {
        if let Ok(value) = std::env::var(name) {
            if !value.trim().is_empty() {
                return Ok(value);
            }
        }
    }
    let metadata_host = std::env::var("GCE_METADATA_HOST")
        .unwrap_or_else(|_| "metadata.google.internal".to_string());
    let url = format!(
        "http://{metadata_host}/computeMetadata/v1/instance/service-accounts/default/token"
    );
    let response = client
        .get(url)
        .header("Metadata-Flavor", "Google")
        .send()
        .map_err(unavailable)?;
    let value: Value = successful(response, "ADC token")?.json().map_err(corrupt)?;
    value
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or(ObjectStoreError::Unauthorized)
}

fn truncate(value: &str, max: usize) -> &str {
    if value.len() <= max {
        value
    } else {
        &value[..value.floor_char_boundary(max)]
    }
}

fn unavailable(error: impl std::fmt::Display) -> ObjectStoreError {
    ObjectStoreError::Unavailable {
        message: error.to_string(),
    }
}

fn corrupt(error: impl std::fmt::Display) -> ObjectStoreError {
    ObjectStoreError::Corrupt {
        message: error.to_string(),
    }
}
