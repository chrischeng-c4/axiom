//! Where certificate code is allowed to live (#3110 AC6).
//!
//! R1 says the generic lifecycle belongs in shared libraries and that Lumen
//! supplies profiles. That is an architectural claim, and architectural claims
//! decay silently: the first copy of an issuance loop into `apps/lumen` compiles,
//! passes every behavioural test, and is only visible to someone who happens to
//! read the right file. So it is checked here, mechanically, against the source
//! tree itself.
//!
//! The check is deliberately a *type* allowlist rather than a keyword denylist.
//! A denylist of "no `rcgen`, no CSR" is a guess about which name the next
//! duplicate will be spelled with; an allowlist of what Lumen's certificate
//! module may contain refuses the one nobody predicted.

use std::path::{Path, PathBuf};

/// Walk up from this crate until the directory holding both `apps` and `libs`.
fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("apps").is_dir() && dir.join("libs").is_dir() {
            return dir;
        }
        assert!(
            dir.pop(),
            "no ancestor of {} holds both apps/ and libs/",
            env!("CARGO_MANIFEST_DIR")
        );
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

/// Every `.rs` file under `dir`.
fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn the_generic_lifecycle_lives_in_the_shared_library() {
    let shared = repo_root().join("libs/service-k8s/src/certificate");
    let present: Vec<String> = rust_files(&shared)
        .iter()
        .filter_map(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_string)
        })
        .collect();

    // Every piece R1 names as generic, by the module that owns it.
    for expected in [
        "state",       // renewal, expiry, rotation ordering
        "issuer",      // the CSR/keypair boundary
        "cas",         // the CA Service requester
        "ephemeral",   // the replaceable in-process signer (R8)
        "projection",  // Secret layout and owner references
        "reconcile",   // retries and the write path
        "status",      // conditions and redaction
        "profile",     // bounds and validation
    ] {
        assert!(
            present.iter().any(|name| name == expected),
            "libs/service-k8s/src/certificate has no `{expected}` module; if it moved into a \
             service, this library is no longer the shared lifecycle: {present:?}"
        );
    }
}

#[test]
fn lumen_owns_certificate_profiles_and_nothing_else() {
    let lumen_cert = repo_root().join("apps/lumen/src/operator/certificate.rs");
    assert!(
        lumen_cert.is_file(),
        "expected Lumen's certificate profiles at {}",
        lumen_cert.display()
    );
    let source = read(&lumen_cert);

    // The allowlist: what a profile module legitimately does is construct
    // `CertificateProfile`s and name the identities this service answers to.
    // Anything that generates a key, speaks to a CA, writes a Secret, or decides
    // when to renew is generic and has a home already.
    let forbidden: [(&str, &str); 8] = [
        ("KeyPair", "generating key material is the shared issuer's job"),
        ("rcgen", "no service links a CSR builder directly"),
        (
            "IssuanceRequest",
            "requesting issuance is the shared reconciler's job",
        ),
        (
            "next_action",
            "renewal and rotation timing is the shared state machine's job",
        ),
        (
            "material_secret",
            "Secret projection is the shared projector's job",
        ),
        (
            "trust_bundle_secret",
            "trust-bundle layout is the shared projector's job",
        ),
        (
            "privateca.googleapis.com",
            "no service talks to CA Service directly",
        ),
        (
            "impl Issuer",
            "a service-local issuer is exactly the duplication R1 forbids",
        ),
    ];
    for (needle, why) in forbidden {
        assert!(
            !source.contains(needle),
            "apps/lumen/src/operator/certificate.rs mentions `{needle}`: {why}"
        );
    }
}

#[test]
fn no_other_lumen_source_file_implements_a_certificate_lifecycle() {
    let root = repo_root();
    let lumen_src = root.join("apps/lumen/src");
    let profiles = root.join("apps/lumen/src/operator/certificate.rs");

    // These names are how a lifecycle gets built, whatever the file is called.
    let lifecycle_markers = [
        "CertificateSigningRequestParams",
        "IssuanceRequest",
        "trust_anchor_pem",
        "next_action(",
    ];

    let mut offenders = Vec::new();
    for path in rust_files(&lumen_src) {
        if path == profiles {
            continue;
        }
        let source = read(&path);
        for marker in lifecycle_markers {
            if source.contains(marker) {
                offenders.push(format!("{} uses `{marker}`", path.display()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "certificate lifecycle machinery appeared under apps/lumen outside the profile module: \
         {offenders:#?}"
    );
}

#[test]
fn the_shared_lifecycle_does_not_know_what_lumen_is() {
    // The direction of the dependency is the property. Shared code that special-
    // cases one service is shared in name only, and the next service to adopt it
    // inherits Lumen's assumptions without being told.
    let shared = repo_root().join("libs/service-k8s/src/certificate");
    let mut offenders = Vec::new();
    for path in rust_files(&shared) {
        let source = read(&path);
        for line in source.lines() {
            let trimmed = line.trim_start();
            // Fixtures name a concrete service because a scope with no instance
            // name is not a scope. The claim is about the code that ships.
            if trimmed.starts_with("#[cfg(test)]") || trimmed.starts_with("mod tests") {
                break;
            }
            // Comments and doc comments may name Lumen: the issues that drove
            // this design are Lumen's, and erasing that provenance would make
            // the code harder to change correctly, not more generic.
            if trimmed.starts_with("//") {
                continue;
            }
            if line.to_lowercase().contains("lumen") {
                offenders.push(format!("{}: {}", path.display(), line.trim()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "shared certificate code branches on Lumen: {offenders:#?}"
    );
}

#[test]
fn terraform_provisions_trust_but_never_issues_a_leaf() {
    // R9, checked where it can actually be checked. A `google_privateca_certificate`
    // resource would make `terraform apply` the renewal mechanism, and it would
    // work — for exactly one lifetime.
    let terraform = repo_root().join("apps/lumen/terraform");
    if !terraform.is_dir() {
        return;
    }
    let mut offenders = Vec::new();
    let mut stack = vec![terraform];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let is_tf = path
                .extension()
                .is_some_and(|ext| ext == "tf" || ext == "hcl");
            if !is_tf {
                continue;
            }
            let source = read(&path);
            if source.contains("resource \"google_privateca_certificate\"") {
                offenders.push(path.display().to_string());
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "Terraform declares leaf certificates, so renewal needs an apply: {offenders:#?}"
    );
}
