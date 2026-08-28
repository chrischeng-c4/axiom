// HANDWRITE-BEGIN gap="missing-generator:logic:certificate-kubernetes-store" tracker="#3221" reason="Kubernetes API backed SecretStore for certificate projection"
//! Kubernetes API implementation of [`super::reconcile::SecretStore`].
//!
//! Projects certificate secrets directly to a Kubernetes cluster using Server-Side
//! Apply (SSA) under a stable field manager (`service-k8s-certificate`).

use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use k8s_openapi::ByteString;
use kube::api::{Api, Patch, PatchParams};
use serde_json::Value;

use super::projection::{
    CERT_KEY, IDENTITY_DIGEST_ANNOTATION, LEAF_ISSUER_ANNOTATION, PRIVATE_KEY_KEY,
    TRUST_BUNDLE_ANNOTATION, TRUST_BUNDLE_KEY,
};
use super::reconcile::{SecretStore, StoreError, StoreErrorKind, StoredSecret};
use super::status::redact;

/// Field manager name used for Server-Side Apply of certificate secrets.
pub const FIELD_MANAGER: &str = "service-k8s-certificate";

/// Required RBAC verbs for KubernetesSecretStore (#3221).
///
/// Three, and all three have a call site. `get` is the read before the apply;
/// `patch` is the apply itself, which SSA sends as an HTTP `PATCH`.
///
/// `create` is the one the wire shape hides. The apiserver authorizes by what
/// the request *does*, not by its method: an apply whose target does not exist
/// yet is a create, and is checked against the `create` verb. Every certificate
/// this store projects is absent exactly once — the first time — so a grant of
/// `get,patch` alone serves every renewal and refuses every bootstrap, with a
/// 403 that reads like a broken cluster rather than like this list.
pub const REQUIRED_RBAC_VERBS: &[&str] = &["create", "get", "patch"];
pub const RBAC_VERBS: &[&str] = REQUIRED_RBAC_VERBS;

/// Lifecycle-owned Secret data keys.
pub const LIFECYCLE_DATA_KEYS: [&str; 3] = [CERT_KEY, PRIVATE_KEY_KEY, TRUST_BUNDLE_KEY];

/// Lifecycle-owned Secret annotation keys.
pub const LIFECYCLE_ANNOTATION_KEYS: [&str; 3] = [
    TRUST_BUNDLE_ANNOTATION,
    LEAF_ISSUER_ANNOTATION,
    IDENTITY_DIGEST_ANNOTATION,
];

/// Lifecycle-owned Secret label keys.
pub const LIFECYCLE_LABEL_KEYS: [&str; 3] = [
    "app.kubernetes.io/name",
    "app.kubernetes.io/managed-by",
    "app.kubernetes.io/component",
];

/// Alias for `StoreError` to maintain module compatibility.
pub type KubernetesStoreError = StoreError;

/// Classify a `kube::Error` into a typed `StoreError`.
pub fn classify_kube_error(err: &kube::Error) -> StoreError {
    match err {
        kube::Error::Api(api_err) => match api_err.code {
            403 => StoreError::forbidden(&api_err.message),
            409 => StoreError::conflict(&api_err.message),
            code if code >= 500 => StoreError::new(StoreErrorKind::Unavailable, &api_err.message),
            code => StoreError::new(StoreErrorKind::Other(code), &api_err.message),
        },
        _ => StoreError::unavailable(redact(&err.to_string())),
    }
}

/// Helper to merge live Secret data/annotations into desired SSA payload
/// as typed Secret `data` (`ByteString`/base64) and check whether the patch is an unchanged no-op.
///
/// Returns `(patch_secret, is_unchanged)`.
pub fn prepare_ssa_patch(
    desired: &Value,
    live: Option<&Secret>,
) -> Result<(Secret, bool), StoreError> {
    let name = desired["metadata"]["name"]
        .as_str()
        .ok_or_else(|| StoreError::malformed("missing metadata.name"))?;
    let namespace = desired["metadata"]["namespace"]
        .as_str()
        .ok_or_else(|| StoreError::malformed("missing metadata.namespace"))?;

    let mut data_map: BTreeMap<String, ByteString> = BTreeMap::new();
    if let Some(sd) = desired["stringData"].as_object() {
        for (k, v) in sd {
            if let Some(text) = v.as_str() {
                data_map.insert(k.clone(), ByteString(text.as_bytes().to_vec()));
            }
        }
    }

    let mut annotations_map: BTreeMap<String, String> = BTreeMap::new();
    if let Some(ann) = desired["metadata"]["annotations"].as_object() {
        for (k, v) in ann {
            if let Some(text) = v.as_str() {
                annotations_map.insert(k.clone(), text.to_string());
            }
        }
    }

    let mut labels_map: BTreeMap<String, String> = BTreeMap::new();
    if let Some(lbl) = desired["metadata"]["labels"].as_object() {
        for (k, v) in lbl {
            if let Some(text) = v.as_str() {
                labels_map.insert(k.clone(), text.to_string());
            }
        }
    }

    let owner_refs: Option<Vec<OwnerReference>> = desired["metadata"]["ownerReferences"]
        .as_array()
        .map(|arr| {
            serde_json::from_value(Value::Array(arr.clone()))
                .map_err(|e| StoreError::malformed(e.to_string()))
        })
        .transpose()?;

    // 1. Merge omitted lifecycle-owned live keys and annotations losslessly
    if let Some(live_secret) = live {
        if let Some(live_data) = &live_secret.data {
            for &key in &LIFECYCLE_DATA_KEYS {
                if !data_map.contains_key(key) {
                    if let Some(bytes) = live_data.get(key) {
                        data_map.insert(key.to_string(), bytes.clone());
                    }
                }
            }
        }

        if let Some(live_ann) = &live_secret.metadata.annotations {
            for &key in &LIFECYCLE_ANNOTATION_KEYS {
                if !annotations_map.contains_key(key) {
                    if let Some(val) = live_ann.get(key) {
                        annotations_map.insert(key.to_string(), val.clone());
                    }
                }
            }
        }
    }

    let patch_secret = Secret {
        type_: Some("Opaque".into()),
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some(namespace.to_string()),
            labels: Some(labels_map.clone()),
            annotations: Some(annotations_map.clone()),
            owner_references: owner_refs.clone(),
            ..Default::default()
        },
        data: Some(data_map.clone()),
        string_data: None,
        ..Default::default()
    };

    // 2. Check if unchanged against live Secret across all lifecycle-owned fields
    let mut unchanged = false;
    if let Some(live_secret) = live {
        unchanged = true;

        if live_secret.metadata.name.as_deref() != Some(name)
            || live_secret.metadata.namespace.as_deref() != Some(namespace)
            || live_secret.type_.as_deref() != Some("Opaque")
        {
            unchanged = false;
        }

        // Owner reference comparison including UID, controller, blockOwnerDeletion
        if unchanged {
            let live_owner_refs = live_secret
                .metadata
                .owner_references
                .as_deref()
                .unwrap_or(&[]);
            if let Some(desired_owners) = &owner_refs {
                for desired_owner in desired_owners {
                    let matched = live_owner_refs.iter().any(|live_owner| {
                        live_owner.api_version == desired_owner.api_version
                            && live_owner.kind == desired_owner.kind
                            && live_owner.name == desired_owner.name
                            && live_owner.uid == desired_owner.uid
                            && live_owner.controller == desired_owner.controller
                            && live_owner.block_owner_deletion == desired_owner.block_owner_deletion
                    });
                    if !matched {
                        unchanged = false;
                        break;
                    }
                }
            } else if !live_owner_refs.is_empty() {
                unchanged = false;
            }
        }

        // Compare lifecycle-owned data bytes
        if unchanged {
            let live_data = live_secret.data.as_ref();
            for &key in &LIFECYCLE_DATA_KEYS {
                let desired_bytes = data_map.get(key);
                let live_bytes = live_data.and_then(|m| m.get(key));
                if desired_bytes != live_bytes {
                    unchanged = false;
                    break;
                }
            }
        }

        // Compare lifecycle-owned annotations
        if unchanged {
            let live_ann = live_secret.metadata.annotations.as_ref();
            for &key in &LIFECYCLE_ANNOTATION_KEYS {
                let desired_val = annotations_map.get(key);
                let live_val = live_ann.and_then(|m| m.get(key));
                if desired_val != live_val {
                    unchanged = false;
                    break;
                }
            }
        }

        // Compare lifecycle-owned labels
        if unchanged {
            let live_labels = live_secret.metadata.labels.as_ref();
            for &key in &LIFECYCLE_LABEL_KEYS {
                let desired_val = labels_map.get(key);
                let live_val = live_labels.and_then(|m| m.get(key));
                if desired_val != live_val {
                    unchanged = false;
                    break;
                }
            }
        }
    }

    Ok((patch_secret, unchanged))
}

/// A Kubernetes API-backed [`SecretStore`].
#[derive(Clone)]
pub struct KubernetesSecretStore {
    client: kube::Client,
}

impl KubernetesSecretStore {
    /// Create a new `KubernetesSecretStore` with the provided `kube::Client`.
    pub fn new(client: kube::Client) -> Self {
        Self { client }
    }
}

impl SecretStore for KubernetesSecretStore {
    fn read<'a>(
        &'a self,
        namespace: &'a str,
        name: &'a str,
    ) -> futures::future::BoxFuture<'a, Result<Option<StoredSecret>, StoreError>> {
        Box::pin(async move {
            let api: Api<Secret> = Api::namespaced(self.client.clone(), namespace);
            match api.get_opt(name).await {
                Ok(None) => Ok(None),
                Ok(Some(secret)) => {
                    let mut data = BTreeMap::new();
                    if let Some(secret_data) = secret.data {
                        for (k, v) in secret_data {
                            data.insert(k, v.0);
                        }
                    }
                    if let Some(string_data) = secret.string_data {
                        for (k, v) in string_data {
                            data.entry(k).or_insert_with(|| v.into_bytes());
                        }
                    }
                    let annotations = secret.metadata.annotations.unwrap_or_default();
                    Ok(Some(StoredSecret { data, annotations }))
                }
                Err(err) => Err(classify_kube_error(&err)),
            }
        })
    }

    fn apply<'a>(
        &'a self,
        object: Value,
    ) -> futures::future::BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(async move {
            let namespace = object["metadata"]["namespace"]
                .as_str()
                .ok_or_else(|| StoreError::malformed("missing metadata.namespace"))?
                .to_string();

            let name = object["metadata"]["name"]
                .as_str()
                .ok_or_else(|| StoreError::malformed("missing metadata.name"))?
                .to_string();

            let api: Api<Secret> = Api::namespaced(self.client.clone(), &namespace);

            let live_secret = match api.get_opt(&name).await {
                Ok(live) => live,
                Err(err) => return Err(classify_kube_error(&err)),
            };

            let (patch_secret, unchanged) = prepare_ssa_patch(&object, live_secret.as_ref())?;

            if unchanged {
                // Pre-PATCH no-op check: leaves resourceVersion unchanged on the API server.
                return Ok(());
            }

            let params = PatchParams::apply(FIELD_MANAGER);
            match api
                .patch(&name, &params, &Patch::Apply(&patch_secret))
                .await
            {
                Ok(_) => Ok(()),
                Err(err) => Err(classify_kube_error(&err)),
            }
        })
    }
}
// HANDWRITE-END
