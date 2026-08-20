// HANDWRITE-BEGIN gap="missing-generator:logic:projected-service-account-token" tracker="#2877" reason="Own the projected `serviceAccountToken` volume/mount pair and the apiserver's expiration floor as one value, so a workload cannot mount a token whose audience and file path disagree with the client that reads it."
//! An audience-bound, short-lived ServiceAccount token, mounted as a file (#2877).
//!
//! A workload that calls a service which authenticates with `TokenReview`
//! needs a credential that is (a) issued to *its own* ServiceAccount, (b)
//! bound to the callee's audience, and (c) short-lived enough that a leaked
//! copy expires before it is useful. Kubernetes issues exactly that through a
//! projected volume, and rotates it in place — the kubelet refreshes the file
//! at roughly 80% of its lifetime, with no pod restart and no notification.
//!
//! Two mistakes are easy to make here, and both are silent:
//!
//! - Mounting the pod's *default* token instead of a projected one. It
//!   authenticates fine against kube-apiserver, so nothing looks broken — but
//!   its audience is the apiserver's, and a callee that checks audience will
//!   reject it. Worse, a callee that does *not* check audience is now
//!   reachable with a token every pod in the cluster already holds.
//! - Rendering the volume in one place and the reader's file path in another.
//!   The two drift, the mount is present, the file the client opens is not,
//!   and the failure surfaces as an authentication error rather than a wiring
//!   one.
//!
//! [`ProjectedServiceAccountToken`] is one value that answers all of it: the
//! volume, the mount, and the absolute path the client reads. Callers keep the
//! audience and the workload list — this type keeps the shape.

use serde_json::{json, Value};

/// The file name inside the mount. `token` is the convention every projected
/// ServiceAccount token in the ecosystem uses, including the one the kubelet
/// mounts by default at `/var/run/secrets/kubernetes.io/serviceaccount`.
pub const DEFAULT_TOKEN_FILE: &str = "token";

/// The shortest lifetime kube-apiserver will issue. `TokenRequest` validation
/// rejects anything below it, so a caller asking for less does not get a
/// tighter token — it gets a workload that never starts.
pub const MINIMUM_EXPIRATION_SECONDS: u32 = 600;

/// `0444` — readable by any UID the pod runs as, writable by none. The
/// kubelet owns the content; nothing in the container should be able to
/// overwrite it and hand the client a credential of its own choosing.
const READ_ONLY_MODE: u32 = 0o444;

/// One projected ServiceAccount token: what to mount, where, for whom.
pub struct ProjectedServiceAccountToken<'a> {
    /// Pod-unique volume name, shared by [`Self::volume`] and [`Self::mount`].
    pub volume_name: &'a str,
    /// Directory the token file appears in. Not the file itself — a projected
    /// volume mounts a directory.
    pub mount_path: &'a str,
    /// File name within `mount_path`; [`DEFAULT_TOKEN_FILE`] unless the callee
    /// has a reason to differ.
    pub file_name: &'a str,
    /// The audience the callee will require. This is the whole point of the
    /// projection: a token minted for `a` is not a credential at `b`.
    pub audience: &'a str,
    /// Requested lifetime. Must be at least [`MINIMUM_EXPIRATION_SECONDS`].
    pub expiration_seconds: u32,
}

impl<'a> ProjectedServiceAccountToken<'a> {
    /// A token at the apiserver's expiration floor, in the conventional file.
    pub fn new(volume_name: &'a str, mount_path: &'a str, audience: &'a str) -> Self {
        Self {
            volume_name,
            mount_path,
            file_name: DEFAULT_TOKEN_FILE,
            audience,
            expiration_seconds: MINIMUM_EXPIRATION_SECONDS,
        }
    }

    /// The `spec.volumes` entry. A `projected` volume with a single
    /// `serviceAccountToken` source — never a Secret, so the credential exists
    /// only in the kubelet's memory and this pod's tmpfs, and is never an
    /// object another workload could read.
    pub fn volume(&self) -> Value {
        json!({
            "name": self.volume_name,
            "projected": {
                "defaultMode": READ_ONLY_MODE,
                "sources": [{
                    "serviceAccountToken": {
                        "audience": self.audience,
                        "expirationSeconds": self.expiration_seconds,
                        "path": self.file_name,
                    }
                }],
            },
        })
    }

    /// The matching `volumeMounts` entry. Read-only: the container has no
    /// business writing here, and `readOnlyRootFilesystem` does not cover a
    /// mounted volume.
    pub fn mount(&self) -> Value {
        json!({
            "name": self.volume_name,
            "mountPath": self.mount_path,
            "readOnly": true,
        })
    }

    /// The absolute path the client opens. Derived from the same two fields
    /// the mount is derived from, so the reader and the manifest cannot drift.
    pub fn file_path(&self) -> String {
        format!(
            "{}/{}",
            self.mount_path.trim_end_matches('/'),
            self.file_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_volume_projects_one_audience_bound_token_and_no_secret() {
        let token = ProjectedServiceAccountToken::new(
            "callee-token",
            "/var/run/secrets/callee.example.com",
            "callee.example.com",
        );
        let volume = token.volume();

        let sources = volume["projected"]["sources"]
            .as_array()
            .expect("projected sources");
        assert_eq!(sources.len(), 1, "one source, one credential: {volume}");
        let sat = &sources[0]["serviceAccountToken"];
        assert_eq!(sat["audience"], "callee.example.com");
        assert_eq!(sat["expirationSeconds"], MINIMUM_EXPIRATION_SECONDS);
        assert_eq!(sat["path"], DEFAULT_TOKEN_FILE);
        assert!(
            volume["secret"].is_null() && volume["configMap"].is_null(),
            "a projected token must not also render a Secret or ConfigMap source: {volume}"
        );
    }

    /// The container may not write over the credential the kubelet rotates,
    /// and neither may a process running as a different UID in the same pod.
    #[test]
    fn the_mount_and_the_file_mode_are_both_read_only() {
        let token = ProjectedServiceAccountToken::new("t", "/var/run/secrets/x", "x");
        assert_eq!(token.mount()["readOnly"], true);
        assert_eq!(token.volume()["projected"]["defaultMode"], 0o444);
    }

    /// The mount and the reader's path are one derivation, not two — this is
    /// the drift the type exists to prevent.
    #[test]
    fn the_file_path_is_the_mount_path_and_survives_a_trailing_slash() {
        let plain = ProjectedServiceAccountToken::new("t", "/var/run/secrets/x", "x");
        assert_eq!(plain.file_path(), "/var/run/secrets/x/token");
        assert_eq!(plain.mount()["mountPath"], "/var/run/secrets/x");

        let trailing = ProjectedServiceAccountToken {
            mount_path: "/var/run/secrets/x/",
            ..ProjectedServiceAccountToken::new("t", "/var/run/secrets/x/", "x")
        };
        assert_eq!(
            trailing.file_path(),
            "/var/run/secrets/x/token",
            "a trailing slash must not produce a doubled separator the client then fails to open"
        );
    }

    /// The volume name is the only link between the two halves; a renderer
    /// that composes them separately has to be able to rely on it.
    #[test]
    fn the_volume_and_the_mount_agree_on_the_name() {
        let token = ProjectedServiceAccountToken::new("callee-token", "/var/run/x", "x");
        assert_eq!(token.volume()["name"], token.mount()["name"]);
        assert_eq!(token.volume()["name"], "callee-token");
    }

    /// A caller may ask for longer than the floor; it may not ask for less and
    /// expect the apiserver to issue it. Pinned so the constant stays the
    /// documented floor rather than an arbitrary default.
    #[test]
    fn the_expiration_floor_is_the_apiservers_and_a_longer_request_renders_verbatim() {
        assert_eq!(MINIMUM_EXPIRATION_SECONDS, 600);
        let long = ProjectedServiceAccountToken {
            expiration_seconds: 3600,
            ..ProjectedServiceAccountToken::new("t", "/var/run/x", "x")
        };
        assert_eq!(
            long.volume()["projected"]["sources"][0]["serviceAccountToken"]["expirationSeconds"],
            3600
        );
    }
}
// HANDWRITE-END
