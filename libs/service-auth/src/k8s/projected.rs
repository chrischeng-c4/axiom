// HANDWRITE-BEGIN gap="missing-generator:logic:projected-token-source" tracker="#2877" reason="A rotation-aware credential reader whose whole contract is what it refuses to do — never cache, never echo the material, never present a token the callee will reject — has no generator primitive."
//! Reading the projected ServiceAccount token a workload was given.
//!
//! The counterpart to the mounted volume: a client that calls an
//! audience-bound service opens this file immediately before each call. That
//! "immediately before" is the whole design.
//!
//! The kubelet rotates the projection in place — it writes the replacement
//! into a new directory and atomically re-points a symlink, at roughly 80% of
//! the token's lifetime. Nothing tells the process. A client that reads once
//! at startup therefore works perfectly for eight minutes and then fails
//! forever, which is the worst possible shape for the failure: it survives
//! every smoke test and breaks in the middle of the night. Re-reading is a
//! file open on tmpfs, so there is nothing to optimise away here.
//!
//! ## What is checked before the token leaves this module
//!
//! The client cannot verify the signature — it holds no key, and it is not the
//! audience. What it *can* do is refuse to send material that is already known
//! to be useless, and say why:
//!
//! - **Missing or unreadable** — the volume was never mounted, or the path
//!   disagrees with the manifest.
//! - **Wrong audience** — a default pod token got mounted instead of a
//!   projected one. This is the case that would otherwise reach the callee and
//!   come back as a bare `401`, sending whoever debugs it to look at RBAC.
//! - **Expired** — the kubelet's refresh did not happen, or the pod was
//!   suspended past the lifetime.
//!
//! Each is a distinct error naming the file and the audience, and none of them
//! carries the token. That is not decoration: a credential that reaches a log
//! line, a Kubernetes Event, or a CR status has left the pod, and the three
//! failures above are exactly the ones a maintainer is tempted to debug by
//! printing the value.

use std::fmt;
use std::path::{Path, PathBuf};

use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;

/// A bearer token that will not print itself.
///
/// `Debug` is the one that matters: `?err`, `{:?}` on a struct that holds a
/// token, and `#[derive(Debug)]` on anything upstream all reach it without
/// anyone deciding to log a credential.
#[derive(Clone)]
pub struct ProjectedToken(String);

impl ProjectedToken {
    /// Wrap material that is already a token.
    ///
    /// Crate-internal on purpose. There are exactly two things in this crate
    /// that hold one — the file this module reads and the TokenRequest
    /// [`super::token_request`] makes — and a public constructor would make
    /// this type a general-purpose string wrapper whose redaction is
    /// decorative rather than a property of where tokens come from.
    pub(crate) fn new(material: String) -> Self {
        Self(material)
    }

    /// The material itself, for the one place that puts it on the wire.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ProjectedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProjectedToken(<redacted>)")
    }
}

impl fmt::Display for ProjectedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted service account token>")
    }
}

/// Why a projected token cannot be presented. Every variant names the file and
/// the audience expected; no variant carries the material or any part of it.
#[derive(Debug)]
pub enum ProjectedTokenError {
    /// The file is absent or unreadable — almost always a volume that was not
    /// mounted, or a path that disagrees with the rendered manifest.
    Unreadable {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The file exists and holds nothing. A projection mid-write looks like
    /// this, and so does a mount of the wrong key.
    Empty { path: PathBuf },
    /// Not a JWT this client can inspect. Reported without the content: a
    /// malformed credential is still a credential.
    Malformed { path: PathBuf },
    /// A valid token minted for someone else — the default pod token is the
    /// usual culprit. Reported without the audiences actually found, which
    /// are claims of an unverified token and would be attacker-chosen text in
    /// a log line.
    WrongAudience { path: PathBuf, expected: String },
    /// Past its expiry. Names the file, because the fix is at the kubelet or
    /// the projection, not at the callee.
    Expired { path: PathBuf },
}

impl fmt::Display for ProjectedTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, source } => write!(
                f,
                "cannot read the projected ServiceAccount token at {}: {source} — is the \
                 projected volume mounted, and does its mountPath match this path?",
                path.display()
            ),
            Self::Empty { path } => write!(
                f,
                "the projected ServiceAccount token at {} is empty",
                path.display()
            ),
            Self::Malformed { path } => write!(
                f,
                "the file at {} is not a readable ServiceAccount token; its contents are \
                 withheld because they may still be a credential",
                path.display()
            ),
            Self::WrongAudience { path, expected } => write!(
                f,
                "the ServiceAccount token at {} was not issued for `{expected}` — this is what \
                 mounting the pod's default token instead of a projected one looks like; add a \
                 `serviceAccountToken` projection with that audience",
                path.display()
            ),
            Self::Expired { path } => write!(
                f,
                "the ServiceAccount token at {} has expired; the kubelet refreshes a projected \
                 token in place, so a persistently expired one means the projection is not \
                 rotating",
                path.display()
            ),
        }
    }
}

impl std::error::Error for ProjectedTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unreadable { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Claims this client is willing to look at. Everything else — issuer, subject,
/// the `kubernetes.io` block — is the callee's business, verified there
/// against a signature this side does not hold.
#[derive(Deserialize)]
struct InspectedClaims {
    #[allow(dead_code)]
    exp: i64,
}

/// A projected ServiceAccount token file, and the audience it must carry.
///
/// Holds no token: every [`Self::read`] goes to the file. See the module note
/// on why caching one here is a bug rather than an optimisation.
pub struct ProjectedTokenFile {
    path: PathBuf,
    audience: String,
}

impl ProjectedTokenFile {
    pub fn new(path: impl Into<PathBuf>, audience: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            audience: audience.into(),
        }
    }

    /// The file this reads, for diagnostics that want to name it.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The audience every token from this file must carry.
    pub fn audience(&self) -> &str {
        &self.audience
    }

    /// The token as it stands right now, or the reason it cannot be presented.
    ///
    /// Call this per request. It is a `read` on tmpfs, and it is the only
    /// thing that makes rotation work.
    pub fn read(&self) -> Result<ProjectedToken, ProjectedTokenError> {
        let raw = std::fs::read_to_string(&self.path).map_err(|source| {
            ProjectedTokenError::Unreadable {
                path: self.path.clone(),
                source,
            }
        })?;
        // Trailing newline: the kubelet does not add one, but a developer
        // writing the file by hand for a local run always does, and a bearer
        // header with a newline in it is a protocol error rather than a 401.
        let token = raw.trim();
        if token.is_empty() {
            return Err(ProjectedTokenError::Empty {
                path: self.path.clone(),
            });
        }
        self.inspect(token)?;
        Ok(ProjectedToken(token.to_string()))
    }

    /// Audience and expiry, without the signature. The key belongs to the
    /// cluster's token issuer and the audience is the callee — this side is
    /// neither, so signature validation here would be theatre. Refusing a
    /// token whose *own claims* already disqualify it is not.
    fn inspect(&self, token: &str) -> Result<(), ProjectedTokenError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.insecure_disable_signature_validation();
        validation.set_audience(&[&self.audience]);
        validation.validate_exp = true;
        // The default 60-second grace, kept deliberately. This check exists to
        // catch a projection that stopped rotating, not to second-guess a
        // clock: the callee holds the authoritative one, and a token a second
        // past expiry by *this* pod's reckoning is one it should still send
        // and let the cluster judge.
        validation.leeway = 60;
        validation.required_spec_claims = ["exp", "aud"].into_iter().map(String::from).collect();

        match jsonwebtoken::decode::<InspectedClaims>(
            token,
            // Unused: signature validation is off above. `jsonwebtoken` still
            // requires a key argument.
            &DecodingKey::from_secret(&[]),
            &validation,
        ) {
            Ok(_) => Ok(()),
            Err(err) => Err(match err.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                    ProjectedTokenError::WrongAudience {
                        path: self.path.clone(),
                        expected: self.audience.clone(),
                    }
                }
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    ProjectedTokenError::Expired {
                        path: self.path.clone(),
                    }
                }
                // Missing `aud` reads as a missing required claim, not as a
                // wrong audience — but for this client they are the same
                // mistake, and the actionable message is the audience one.
                jsonwebtoken::errors::ErrorKind::MissingRequiredClaim(claim) if claim == "aud" => {
                    ProjectedTokenError::WrongAudience {
                        path: self.path.clone(),
                        expected: self.audience.clone(),
                    }
                }
                _ => ProjectedTokenError::Malformed {
                    path: self.path.clone(),
                },
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::Serialize;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// A recognisable string, so a test can assert it appears nowhere in an
    /// error rendering rather than asserting on the shape of the message.
    const CANARY: &str = "canary-service-account-token-must-never-be-printed";

    #[derive(Serialize)]
    struct Claims {
        aud: Vec<String>,
        exp: i64,
        sub: String,
        /// Carries the canary into the token's own payload, so a test proves
        /// the *material* is withheld and not merely that the wrapper's
        /// `Display` was called.
        jti: String,
    }

    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after the epoch")
            .as_secs() as i64
    }

    /// A syntactically real JWT. Signed with a throwaway HMAC key — the reader
    /// under test never checks the signature, and building one this way keeps
    /// the fixture honest about JWT framing.
    fn token_for(audience: &str, exp: i64) -> String {
        encode(
            &Header::new(Algorithm::HS256),
            &Claims {
                aud: vec![audience.to_string()],
                exp,
                sub: "system:serviceaccount:ops:caller".to_string(),
                jti: CANARY.to_string(),
            },
            &EncodingKey::from_secret(b"irrelevant"),
        )
        .expect("encode fixture token")
    }

    struct Mount {
        dir: PathBuf,
    }

    impl Mount {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "service-auth-projected-{tag}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("create fixture mount");
            Self { dir }
        }

        fn path(&self) -> PathBuf {
            self.dir.join("token")
        }

        fn write(&self, contents: &str) {
            std::fs::write(self.path(), contents).expect("write fixture token");
        }

        fn file(&self, audience: &str) -> ProjectedTokenFile {
            ProjectedTokenFile::new(self.path(), audience)
        }
    }

    impl Drop for Mount {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn a_token_with_the_expected_audience_reads_back_verbatim() {
        let mount = Mount::new("happy");
        let token = token_for("callee.example.com", now() + 600);
        mount.write(&token);

        let read = mount
            .file("callee.example.com")
            .read()
            .expect("a current, correctly-scoped token is presentable");
        assert_eq!(read.expose(), token);
    }

    /// The rotation contract, and the reason nothing is cached: the kubelet
    /// replaces the file underneath a running process, and the next request
    /// must go out with the replacement.
    #[test]
    fn a_rotated_file_is_picked_up_by_the_next_read_without_a_restart() {
        let mount = Mount::new("rotation");
        let first = token_for("callee.example.com", now() + 600);
        mount.write(&first);
        let file = mount.file("callee.example.com");

        assert_eq!(file.read().expect("first read").expose(), first);

        // Same process, same `ProjectedTokenFile`, new material — exactly what
        // the kubelet does at ~80% of the lifetime.
        let second = token_for("callee.example.com", now() + 1200);
        assert_ne!(first, second, "the fixture must actually rotate");
        mount.write(&second);

        assert_eq!(
            file.read().expect("second read").expose(),
            second,
            "a cached token would have kept presenting the old one until it expired"
        );
    }

    #[test]
    fn a_trailing_newline_is_not_part_of_the_credential() {
        let mount = Mount::new("newline");
        let token = token_for("callee.example.com", now() + 600);
        mount.write(&format!("{token}\n"));

        assert_eq!(
            mount
                .file("callee.example.com")
                .read()
                .expect("read")
                .expose(),
            token
        );
    }

    #[test]
    fn a_missing_mount_is_an_actionable_error_naming_the_path() {
        let file = ProjectedTokenFile::new("/nonexistent/projected/token", "callee.example.com");
        let err = file.read().expect_err("a missing file cannot be presented");
        assert!(matches!(err, ProjectedTokenError::Unreadable { .. }), "{err:?}");
        let rendered = err.to_string();
        assert!(rendered.contains("/nonexistent/projected/token"), "{rendered}");
        assert!(rendered.contains("projected volume"), "{rendered}");
    }

    #[test]
    fn an_empty_file_is_refused_rather_than_sent_as_an_empty_bearer() {
        let mount = Mount::new("empty");
        mount.write("   \n");
        let err = mount.file("callee.example.com").read().expect_err("empty");
        assert!(matches!(err, ProjectedTokenError::Empty { .. }), "{err:?}");
    }

    /// The case worth catching locally: the pod's *default* token. It is a
    /// perfectly valid credential minted for kube-apiserver, so the callee
    /// answers `401` and the operator goes looking at RBAC.
    #[test]
    fn a_default_pod_token_is_refused_here_rather_than_at_the_callee() {
        let mount = Mount::new("audience");
        mount.write(&token_for(
            "https://kubernetes.default.svc.cluster.local",
            now() + 3600,
        ));

        let err = mount
            .file("callee.example.com")
            .read()
            .expect_err("wrong audience");
        assert!(
            matches!(err, ProjectedTokenError::WrongAudience { .. }),
            "{err:?}"
        );
        let rendered = err.to_string();
        assert!(rendered.contains("callee.example.com"), "{rendered}");
        assert!(rendered.contains("serviceAccountToken"), "{rendered}");
    }

    #[test]
    fn an_expired_token_is_refused_and_the_message_points_at_rotation() {
        let mount = Mount::new("expired");
        mount.write(&token_for("callee.example.com", now() - 3600));

        let err = mount
            .file("callee.example.com")
            .read()
            .expect_err("expired");
        assert!(matches!(err, ProjectedTokenError::Expired { .. }), "{err:?}");
        assert!(err.to_string().contains("rotating"), "{err}");
    }

    #[test]
    fn a_file_that_is_not_a_token_at_all_is_malformed() {
        let mount = Mount::new("garbage");
        mount.write("not-a-jwt");
        let err = mount.file("callee.example.com").read().expect_err("garbage");
        assert!(matches!(err, ProjectedTokenError::Malformed { .. }), "{err:?}");
    }

    /// AC4's redaction requirement, asserted against the material rather than
    /// against the wording: no rendering of a token or of any failure to read
    /// one may contain the bytes that were on disk.
    #[test]
    fn no_rendering_of_a_token_or_its_failures_contains_the_material() {
        let mount = Mount::new("redaction");

        let current = token_for("callee.example.com", now() + 600);
        mount.write(&current);
        let token = mount
            .file("callee.example.com")
            .read()
            .expect("read the good token");
        for rendered in [format!("{token}"), format!("{token:?}")] {
            assert!(
                !rendered.contains(CANARY) && !rendered.contains(&current),
                "a token printed itself: {rendered}"
            );
        }

        // Every failure path, including the ones whose input is a valid token.
        let cases = [
            token_for("someone.else.example.com", now() + 600),
            token_for("callee.example.com", now() - 600),
            format!("not-a-jwt-{CANARY}"),
        ];
        for material in cases {
            mount.write(&material);
            let err = mount
                .file("callee.example.com")
                .read()
                .expect_err("each case is a refusal");
            for rendered in [err.to_string(), format!("{err:?}")] {
                assert!(
                    !rendered.contains(CANARY) && !rendered.contains(&material),
                    "an error echoed the credential it refused: {rendered}"
                );
            }
        }
    }
}
// HANDWRITE-END
