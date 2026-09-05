use std::future::Future;

use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{primitives::ByteStream, Client};

use crate::{
    validate_key, Object, ObjectMeta, ObjectStore, ObjectStoreError, ObjectVersion, PutCondition,
    Result,
};

/// S3-compatible object store. Authentication always comes from the ambient
/// AWS credential chain, such as an instance role or IRSA.
#[derive(Clone, Debug)]
pub struct S3ObjectStore {
    bucket: String,
    prefix: String,
    region: Option<String>,
    endpoint: Option<String>,
}

impl S3ObjectStore {
    pub fn new(
        bucket: impl Into<String>,
        prefix: impl Into<String>,
        region: Option<String>,
        endpoint: Option<String>,
    ) -> Result<Self> {
        let bucket = bucket.into();
        if bucket.trim().is_empty() || bucket.contains('/') {
            return Err(ObjectStoreError::InvalidKey { key: bucket });
        }
        Ok(Self {
            bucket,
            prefix: prefix.into().trim_matches('/').to_string(),
            region: region.or_else(|| endpoint.as_ref().map(|_| "us-east-1".to_string())),
            endpoint,
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

    async fn client(region: Option<String>, endpoint: Option<String>) -> Result<Client> {
        let mut loader = aws_config::defaults(BehaviorVersion::latest());
        if let Some(region) = region {
            loader = loader.region(Region::new(region));
        }
        let shared = loader.load().await;
        let mut builder = aws_sdk_s3::config::Builder::from(&shared);
        if let Some(endpoint) = endpoint {
            builder = builder.endpoint_url(endpoint).force_path_style(true);
        }
        Ok(Client::from_conf(builder.build()))
    }

    fn run<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(Client) -> std::pin::Pin<Box<dyn Future<Output = Result<T>> + Send>>
            + Send
            + 'static,
        T: Send + 'static,
    {
        let region = self.region.clone();
        let endpoint = self.endpoint.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| ObjectStoreError::Unavailable {
                    message: format!("build S3 runtime: {error}"),
                })?;
            runtime.block_on(async move {
                let client = Self::client(region, endpoint).await?;
                operation(client).await
            })
        })
        .join()
        .map_err(|_| ObjectStoreError::Unavailable {
            message: "S3 worker thread panicked".to_string(),
        })?
    }

    fn map_error(key: &str, error: impl std::fmt::Display) -> ObjectStoreError {
        let message = error.to_string();
        if message.contains("PreconditionFailed") || message.contains("status code: 412") {
            ObjectStoreError::PreconditionFailed {
                key: key.to_string(),
            }
        } else if message.contains("NoSuchKey")
            || message.contains("NotFound")
            || message.contains("status code: 404")
        {
            ObjectStoreError::NotFound {
                key: key.to_string(),
            }
        } else if message.contains("AccessDenied")
            || message.contains("InvalidAccessKeyId")
            || message.contains("status code: 401")
            || message.contains("status code: 403")
        {
            ObjectStoreError::Unauthorized
        } else {
            ObjectStoreError::Unavailable { message }
        }
    }

    fn version(version_id: Option<&str>, etag: Option<&str>) -> ObjectVersion {
        ObjectVersion::new(
            version_id
                .or(etag)
                .unwrap_or("unversioned")
                .trim_matches('"'),
        )
    }
}

impl ObjectStore for S3ObjectStore {
    fn put(
        &self,
        key: &str,
        bytes: &[u8],
        content_type: &str,
        condition: PutCondition,
    ) -> Result<ObjectMeta> {
        let full_key = self.full_key(key)?;
        let result_key = key.to_string();
        let error_key = result_key.clone();
        let bucket = self.bucket.clone();
        let body = bytes.to_vec();
        let content_type = content_type.to_string();
        self.run(move |client| {
            Box::pin(async move {
                let mut request = client
                    .put_object()
                    .bucket(&bucket)
                    .key(&full_key)
                    .content_type(&content_type)
                    .body(ByteStream::from(body.clone()));
                request = match condition {
                    PutCondition::Any => request,
                    PutCondition::IfAbsent => request.if_none_match("*"),
                    PutCondition::IfVersion(version) => request.if_match(version.as_str()),
                };
                let response = request
                    .send()
                    .await
                    .map_err(|error| Self::map_error(&error_key, error))?;
                Ok(ObjectMeta {
                    key: result_key,
                    size: body.len() as u64,
                    content_type,
                    version: Self::version(response.version_id(), response.e_tag()),
                    etag: response.e_tag().map(str::to_string),
                    updated: None,
                })
            })
        })
    }

    fn get(&self, key: &str) -> Result<Object> {
        let full_key = self.full_key(key)?;
        let result_key = key.to_string();
        let error_key = result_key.clone();
        let bucket = self.bucket.clone();
        self.run(move |client| {
            Box::pin(async move {
                let response = client
                    .get_object()
                    .bucket(&bucket)
                    .key(&full_key)
                    .send()
                    .await
                    .map_err(|error| Self::map_error(&error_key, error))?;
                let size = response.content_length().unwrap_or_default().max(0) as u64;
                let content_type = response
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let version = Self::version(response.version_id(), response.e_tag());
                let etag = response.e_tag().map(str::to_string);
                let updated = response.last_modified().map(ToString::to_string);
                let bytes = response
                    .body
                    .collect()
                    .await
                    .map_err(|error| ObjectStoreError::Unavailable {
                        message: format!("read S3 object body: {error}"),
                    })?
                    .into_bytes()
                    .to_vec();
                Ok(Object {
                    meta: ObjectMeta {
                        key: result_key,
                        size,
                        content_type,
                        version,
                        etag,
                        updated,
                    },
                    bytes,
                })
            })
        })
    }

    fn head(&self, key: &str) -> Result<ObjectMeta> {
        let full_key = self.full_key(key)?;
        let result_key = key.to_string();
        let error_key = result_key.clone();
        let bucket = self.bucket.clone();
        self.run(move |client| {
            Box::pin(async move {
                let response = client
                    .head_object()
                    .bucket(&bucket)
                    .key(&full_key)
                    .send()
                    .await
                    .map_err(|error| Self::map_error(&error_key, error))?;
                Ok(ObjectMeta {
                    key: result_key,
                    size: response.content_length().unwrap_or_default().max(0) as u64,
                    content_type: response
                        .content_type()
                        .unwrap_or("application/octet-stream")
                        .to_string(),
                    version: Self::version(response.version_id(), response.e_tag()),
                    etag: response.e_tag().map(str::to_string),
                    updated: response.last_modified().map(ToString::to_string),
                })
            })
        })
    }

    fn list(&self, prefix: &str) -> Result<Vec<ObjectMeta>> {
        let full_prefix = if prefix.trim_matches('/').is_empty() {
            self.prefix.clone()
        } else {
            self.full_key(prefix.trim_matches('/'))?
        };
        let root_prefix = self.prefix.clone();
        let bucket = self.bucket.clone();
        self.run(move |client| {
            Box::pin(async move {
                let mut continuation = None::<String>;
                let mut objects = Vec::new();
                loop {
                    let mut request = client
                        .list_objects_v2()
                        .bucket(&bucket)
                        .prefix(&full_prefix);
                    if let Some(token) = continuation.as_deref() {
                        request = request.continuation_token(token);
                    }
                    let response = request
                        .send()
                        .await
                        .map_err(|error| Self::map_error(&full_prefix, error))?;
                    for object in response.contents() {
                        let Some(full_key) = object.key() else {
                            continue;
                        };
                        let key = if root_prefix.is_empty() {
                            full_key.to_string()
                        } else {
                            full_key
                                .strip_prefix(&format!("{root_prefix}/"))
                                .ok_or_else(|| ObjectStoreError::Corrupt {
                                    message: format!(
                                        "S3 returned key outside configured prefix: {full_key}"
                                    ),
                                })?
                                .to_string()
                        };
                        let etag = object.e_tag().map(str::to_string);
                        objects.push(ObjectMeta {
                            key,
                            size: object.size().unwrap_or_default().max(0) as u64,
                            content_type: "application/octet-stream".to_string(),
                            version: Self::version(None, object.e_tag()),
                            etag,
                            updated: object.last_modified().map(ToString::to_string),
                        });
                    }
                    continuation = response.next_continuation_token().map(str::to_string);
                    if !response.is_truncated().unwrap_or(false) {
                        break;
                    }
                }
                objects.sort_by(|left, right| left.key.cmp(&right.key));
                Ok(objects)
            })
        })
    }

    fn delete(&self, key: &str) -> Result<()> {
        let full_key = self.full_key(key)?;
        let error_key = key.to_string();
        let bucket = self.bucket.clone();
        self.run(move |client| {
            Box::pin(async move {
                client
                    .delete_object()
                    .bucket(&bucket)
                    .key(&full_key)
                    .send()
                    .await
                    .map_err(|error| Self::map_error(&error_key, error))?;
                Ok(())
            })
        })
    }
}
