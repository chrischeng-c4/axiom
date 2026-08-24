//! Deterministic oracle for the Lumen release supply chain.
use serde_yaml::Value as Yaml;
use std::{
    collections::BTreeMap, fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub code: &'static str,
    pub detail: String,
}

const CHECKOUT: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
const ATTEST: &str = "actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6";
const SBOM: &str = "anchore/sbom-action@e22c389904149dbc22b58101806040fa8d37a610";
const BUILD_PUSH: &str = "docker/build-push-action@53b7df96c91f9c12dcc8a07bcb9ccacbed38856a";
const RELEASE_IF: &str =
    "github.event_name == 'push' && startsWith(github.ref, 'refs/tags/lumen@')";
macro_rules! require {
    ($condition:expr, $code:expr, $detail:expr) => {
        if !$condition {
            return Err(Finding {
                code: $code,
                detail: $detail.into(),
            });
        }
    };
}
struct Inputs {
    workflow: String,
    verifier: String,
    kind: String,
    docs: String,
    dockerfile: String,
    cargo: String,
    verifier_mode: u32,
    rendered: String,
}

#[rustfmt::skip]
fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().into() }
#[rustfmt::skip]
fn key(value: &str) -> Yaml { Yaml::String(value.into()) }
#[rustfmt::skip]
fn field<'a>(value: &'a Yaml, name: &str) -> Option<&'a Yaml> { value.as_mapping()?.get(&key(name)) }
#[rustfmt::skip]
fn job<'a>(workflow: &'a Yaml, name: &str) -> Option<&'a Yaml> { field(field(workflow, "jobs")?, name) }
#[rustfmt::skip]
fn steps(job: &Yaml) -> Vec<&Yaml> { field(job, "steps").and_then(Yaml::as_sequence).map_or_else(Vec::new, |v| v.iter().collect()) }
#[rustfmt::skip]
fn named<'a>(job: &'a Yaml, name: &str) -> Option<&'a Yaml> {
    let found: Vec<_> = steps(job).into_iter().filter(|s| field(s, "name").and_then(Yaml::as_str) == Some(name)).collect();
    if found.len() == 1 { Some(found[0]) } else { None }
}
#[rustfmt::skip]
fn using<'a>(job: &'a Yaml, action: &str) -> Vec<&'a Yaml> { steps(job).into_iter().filter(|s| field(s, "uses").and_then(Yaml::as_str) == Some(action)).collect() }
#[rustfmt::skip]
fn strings(value: Option<&Yaml>) -> Vec<&str> { match value { Some(Yaml::String(v)) => vec![v], Some(Yaml::Sequence(v)) => v.iter().filter_map(Yaml::as_str).collect(), _ => Vec::new() } }
#[rustfmt::skip]
fn string_map(value: Option<&Yaml>) -> Option<BTreeMap<String, String>> { value?.as_mapping()?.iter().map(|(k, v)| Some((k.as_str()?.into(), v.as_str()?.into()))).collect() }
#[rustfmt::skip]
fn with_str<'a>(step: &'a Yaml, name: &str) -> Option<&'a str> { field(field(step, "with")?, name)?.as_str() }
#[rustfmt::skip]
fn run(step: &Yaml) -> &str { field(step, "run").and_then(Yaml::as_str).unwrap_or("") }
#[rustfmt::skip]
fn enabled(step: &Yaml) -> bool { field(step, "if").is_none() && field(step, "continue-on-error").is_none() }
fn shell_fn<'a>(source: &'a str, name: &str) -> &'a str {
    let marker = format!("{name}() {{");
    source
        .split_once(&marker)
        .and_then(|(_, tail)| tail.split_once("\n}\n"))
        .map_or("", |v| v.0)
}
fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .and_then(|(_, tail)| tail.split_once(end))
        .map_or("", |v| v.0)
}

#[rustfmt::skip]
fn validate(input: &Inputs) -> Result<(), Finding> {
    let workflow: Yaml = serde_yaml::from_str(&input.workflow)
        .map_err(|e| Finding { code: "YAML_PARSE", detail: e.to_string() })?;
    require!(field(&workflow, "permissions").is_none(), "TOP_PERMISSIONS", "workflow permissions must be absent");
    let events = field(&workflow, "on").and_then(Yaml::as_mapping);
    require!(events.is_some() && events.unwrap().len() == 2, "EVENT_ROUTE", "only push tags and workflow_dispatch are allowed");
    let push = events.unwrap().get(&key("push"));
    require!(strings(push.and_then(|v| field(v, "tags"))) == ["lumen@*"], "EVENT_ROUTE", "tag discovery route changed");
    require!(events.unwrap().get(&key("workflow_dispatch")).is_some_and(Yaml::is_null), "MANUAL_ROUTE", "manual release inputs are forbidden");

    let job_names = ["build", "release-identity", "draft-release", "ghcr-image-and-attest", "verify-artifacts", "kind-amd64", "kind-arm64", "publish-release"];
    let jobs = field(&workflow, "jobs").and_then(Yaml::as_mapping);
    require!(jobs.is_some() && jobs.unwrap().len() == job_names.len() && job_names.iter().all(|n| job(&workflow, n).is_some()), "JOB_GRAPH", "job inventory changed");
    let graph = [
        ("build", vec![]), ("release-identity", vec!["build"]),
        ("draft-release", vec!["build", "release-identity"]),
        ("ghcr-image-and-attest", vec!["build", "release-identity", "draft-release"]),
        ("verify-artifacts", vec!["release-identity", "draft-release", "ghcr-image-and-attest"]),
        ("kind-amd64", vec!["release-identity", "ghcr-image-and-attest", "verify-artifacts"]),
        ("kind-arm64", vec!["release-identity", "ghcr-image-and-attest", "verify-artifacts"]),
        ("publish-release", vec!["release-identity", "draft-release", "ghcr-image-and-attest", "verify-artifacts", "kind-amd64", "kind-arm64"]),
    ];
    for (name, expected) in graph {
        require!(strings(field(job(&workflow, name).unwrap(), "needs")) == expected, "JOB_GRAPH", format!("{name} needs changed"));
    }
    let permissions = [
        ("build", &[("contents", "read")][..]),
        ("release-identity", &[("contents", "read"), ("pull-requests", "read")]),
        ("draft-release", &[("contents", "write")]),
        ("ghcr-image-and-attest", &[("attestations", "write"), ("contents", "read"), ("id-token", "write"), ("packages", "write")]),
        ("verify-artifacts", &[("attestations", "read"), ("contents", "read"), ("packages", "read")]),
        ("kind-amd64", &[("contents", "read"), ("packages", "read")]),
        ("kind-arm64", &[("contents", "read"), ("packages", "read")]),
        ("publish-release", &[("contents", "write"), ("packages", "write")]),
    ];
    for (name, pairs) in permissions {
        let expected = pairs.iter().map(|(k, v)| ((*k).to_string(), (*v).to_string())).collect();
        let actual = string_map(field(job(&workflow, name).unwrap(), "permissions"));
        let code = match name { "ghcr-image-and-attest" => "IMAGE_PERMISSIONS", "publish-release" => "PUBLISH_PERMISSIONS", _ => "JOB_PERMISSIONS" };
        require!(actual == Some(expected), code, format!("{name} permissions changed"));
    }
    for name in ["release-identity", "draft-release", "ghcr-image-and-attest"] {
        require!(field(job(&workflow, name).unwrap(), "if").and_then(Yaml::as_str) == Some(RELEASE_IF), "EVENT_ROUTE", format!("{name} release gate changed"));
    }
    for name in ["build", "verify-artifacts", "kind-amd64", "kind-arm64", "publish-release"] {
        require!(field(job(&workflow, name).unwrap(), "if").is_none(), "EVENT_ROUTE", format!("{name} bypass condition added"));
    }
    for name in job_names {
        let checkout = using(job(&workflow, name).unwrap(), CHECKOUT);
        require!(checkout.len() == 1 && with_str(checkout[0], "ref") == Some("${{ github.sha }}") && field(field(checkout[0], "with").unwrap(), "fetch-depth").and_then(Yaml::as_i64) == Some(0), "CHECKOUT_IDENTITY", format!("{name} checkout is not exact/full"));
    }

    let build = job(&workflow, "build").unwrap();
    let targets = field(field(field(build, "strategy").unwrap(), "matrix").unwrap(), "include").and_then(Yaml::as_sequence).unwrap();
    let targets: Vec<_> = targets.iter().filter_map(|v| field(v, "target").and_then(Yaml::as_str)).collect();
    require!(targets == ["aarch64-apple-darwin", "x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu", "x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl"], "BINARY_TARGETS", "binary target matrix changed");
    let toolchain = using(build, "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c");
    require!(toolchain.len() == 1 && with_str(toolchain[0], "toolchain") == Some("stable"), "ACTION_PIN", "stable Rust toolchain input changed");
    let expected_actions = [
        ("actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1", 8), ("dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c # stable", 1),
        ("Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6 # v2.9.2", 1), ("actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2", 2),
        ("actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093 # v4.3.0", 4), ("softprops/action-gh-release@3d0d9888cb7fd7b750713d6e236d1fcb99157228 # v3.0.2", 1),
        ("docker/setup-qemu-action@c7c53464625b32c7a7e944ae62b3e17d2b600130 # v3.7.0", 1), ("docker/setup-buildx-action@8d2750c68a42422c14e847fe6c8ac0403b4cbd6f # v3.12.0", 2),
        ("docker/login-action@c94ce9fb468520275223c153574b00df6fe4bcc9 # v3.7.0", 2), ("docker/build-push-action@53b7df96c91f9c12dcc8a07bcb9ccacbed38856a # v7.3.0", 1),
        ("sigstore/cosign-installer@6f9f17788090df1f26f669e9d70d6ae9567deba6 # v4.1.2", 2), ("actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6 # v4.2.2", 3),
        ("anchore/sbom-action@e22c389904149dbc22b58101806040fa8d37a610 # v0.24.0", 2),
    ];
    let mut actions = BTreeMap::new();
    for line in input.workflow.lines().map(str::trim) {
        if let Some(value) = line.strip_prefix("- uses: ").or_else(|| line.strip_prefix("uses: ")) { *actions.entry(value.to_string()).or_insert(0usize) += 1; }
    }
    require!(actions == expected_actions.into_iter().map(|(k, v)| (k.into(), v)).collect(), "ACTION_PIN", "action pin, comment, or count changed");

    let identity = run(named(job(&workflow, "release-identity").unwrap(), "Validate release identity and source ancestry").unwrap());
    for needle in ["^lumen@[0-9]+\\.[0-9]+\\.[0-9]+$", "cargo_version=", "\"$version\" != \"$cargo_version\"", "git cat-file -t \"$GITHUB_REF\"", "${GITHUB_REF}^{commit}", "\"$tag_commit\" != \"$GITHUB_SHA\"", "git merge-base --is-ancestor", "gh api --paginate", ".merge_commit_sha == $sha", "expected exactly 1 merged PR"] {
        require!(identity.contains(needle), "SOURCE_IDENTITY", format!("missing source proof: {needle}"));
    }
    let package = named(build, "Package artifact").map(run).unwrap_or("");
    require!(package.contains("cd out") && package.contains("shasum -a 256 \"${name}.tar.gz\" > \"${name}.tar.gz.sha256\"") && package.contains("sha256sum \"${name}.tar.gz\" > \"${name}.tar.gz.sha256\""), "BINARY_CHECKSUM", "archive sidecars must carry their filenames");
    let image = job(&workflow, "ghcr-image-and-attest").unwrap();
    let downloads = using(image, "actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093");
    require!(downloads.len() == 2 && with_str(downloads[0], "name") == Some("lumen-x86_64-unknown-linux-musl") && with_str(downloads[0], "path") == Some("pkg/amd64") && with_str(downloads[1], "name") == Some("lumen-aarch64-unknown-linux-musl") && with_str(downloads[1], "path") == Some("pkg/arm64"), "IMAGE_INPUTS", "image must consume the two exact Actions artifacts");
    let tags = named(image, "Resolve image repository and candidate tag").map(run).unwrap_or("");
    require!(tags.contains("image_repo=ghcr.io/chrischeng-c4/lumen") && tags.contains("candidate_tag=release-candidate-${{ github.run_id }}-${{ github.run_attempt }}"), "IMAGE_INPUTS", "run-scoped candidate identity changed");
    let push = using(image, BUILD_PUSH);
    require!(push.len() == 1, "IMAGE_BUILD", "exactly one image build is required");
    let push = push[0];
    require!(with_str(push, "platforms") == Some("linux/amd64,linux/arm64"), "IMAGE_PLATFORMS", "image platforms changed");
    require!(field(field(push, "with").unwrap(), "provenance").and_then(Yaml::as_bool) == Some(false) && field(field(push, "with").unwrap(), "sbom").and_then(Yaml::as_bool) == Some(false), "BUILDKIT_POLICY", "implicit descriptors must stay disabled");
    require!(with_str(push, "build-args").is_some_and(|v| v.lines().collect::<Vec<_>>() == ["SOURCE=staged"]), "STAGED_SOURCE", "workflow must select only staged binaries");
    require!(with_str(push, "tags") == Some("${{ steps.tags.outputs.image_repo }}:${{ steps.tags.outputs.candidate_tag }}") && with_str(push, "labels").is_some_and(|v| ["org.opencontainers.image.source=https://github.com/${{ github.repository }}", "org.opencontainers.image.revision=${{ needs.release-identity.outputs.commit }}", "org.opencontainers.image.version=${{ needs.release-identity.outputs.version }}", "org.opencontainers.image.url=https://github.com/${{ github.repository }}/actions/runs/${{ github.run_id }}"].iter().all(|line| v.lines().any(|got| got == *line))), "IMAGE_LABELS", "candidate tag or OCI labels changed");
    require!(!steps(image).iter().any(|s| run(s).contains("gh release download")), "PUBLIC_RELEASE_INPUT", "image build must use Actions artifacts");
    let inspect = named(image, "Inspect image index and extract platform child digests").map(run).unwrap_or("");
    for needle in [".manifests | type == \"array\" and length == 2", "(.platform | has(\"variant\") | not)", "(.digest | type == \"string\")", "sort == [\"amd64\", \"arm64\"]", "unique | length == 2", "digests must be pairwise distinct"] {
        require!(inspect.contains(needle), "IMAGE_INDEX", format!("image index proof missing: {needle}"));
    }
    let checksum = named(image, "Verify musl checksums and extract binaries").map(run).unwrap_or("");
    for target in ["x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl"] {
        require!(checksum.contains(&format!("sha256sum -c lumen-{target}.tar.gz.sha256")) && checksum.contains(&format!("shasum -a 256 -c lumen-{target}.tar.gz.sha256")), "MUSL_CHECKSUM", format!("{target} checksum proof missing"));
    }
    let sign = named(image, "Sign root image index with keyless cosign");
    require!(sign.is_some() && enabled(sign.unwrap()) && run(sign.unwrap()).matches("cosign sign --yes").count() == 1 && run(sign.unwrap()).contains("steps.push.outputs.digest"), "ROOT_SIGNATURE", "root signature changed or disabled");
    let signature_count: usize = jobs.unwrap().values().flat_map(steps).map(|step| run(step).matches("cosign sign --yes").count()).sum();
    require!(signature_count == 1, "ROOT_SIGNATURE", "exactly one root signature is allowed");
    let provenance = named(image, "Attest root index provenance");
    require!(provenance.is_some() && enabled(provenance.unwrap()) && field(provenance.unwrap(), "uses").and_then(Yaml::as_str) == Some(ATTEST), "ROOT_PROVENANCE", "root provenance step missing or disabled");
    let provenance = provenance.unwrap();
    require!(with_str(provenance, "subject-name") == Some("${{ steps.tags.outputs.image_repo }}") && with_str(provenance, "subject-digest") == Some("${{ steps.push.outputs.digest }}") && field(field(provenance, "with").unwrap(), "push-to-registry").and_then(Yaml::as_bool) == Some(true) && field(field(provenance, "with").unwrap(), "create-storage-record").and_then(Yaml::as_bool) == Some(false) && with_str(provenance, "sbom-path").is_none(), "ROOT_PROVENANCE", "root provenance inputs changed");
    for (arch, digest, path) in [("amd64", "${{ steps.platform_digests.outputs.amd64_digest }}", "spdx-amd64.json"), ("arm64", "${{ steps.platform_digests.outputs.arm64_digest }}", "spdx-arm64.json")] {
        let generator = named(image, &format!("Generate SPDX SBOM for linux/{arch}"));
        let expected_image = format!("${{{{ steps.tags.outputs.image_repo }}}}@{digest}");
        require!(generator.is_some() && enabled(generator.unwrap()) && field(generator.unwrap(), "uses").and_then(Yaml::as_str) == Some(SBOM) && with_str(generator.unwrap(), "image") == Some(expected_image.as_str()) && with_str(generator.unwrap(), "format") == Some("spdx-json") && with_str(generator.unwrap(), "syft-version") == Some("v1.51.0") && with_str(generator.unwrap(), "output-file") == Some(path) && ["upload-artifact", "upload-release-assets", "dependency-snapshot"].iter().all(|name| field(field(generator.unwrap(), "with").unwrap(), name).and_then(Yaml::as_bool) == Some(false)), "PLATFORM_SBOM", format!("{arch} SPDX generation changed"));
        let attest = named(image, &format!("Attest linux/{arch} SBOM"));
        require!(attest.is_some() && enabled(attest.unwrap()) && field(attest.unwrap(), "uses").and_then(Yaml::as_str) == Some(ATTEST) && with_str(attest.unwrap(), "subject-name") == Some("${{ steps.tags.outputs.image_repo }}") && with_str(attest.unwrap(), "subject-digest") == Some(digest) && with_str(attest.unwrap(), "sbom-path") == Some(path) && field(field(attest.unwrap(), "with").unwrap(), "push-to-registry").and_then(Yaml::as_bool) == Some(true) && field(field(attest.unwrap(), "with").unwrap(), "create-storage-record").and_then(Yaml::as_bool) == Some(false), "PLATFORM_SBOM", format!("{arch} SBOM attestation changed"));
    }
    require!(using(image, SBOM).len() == 2 && using(image, ATTEST).len() == 3, "ATTESTATION_COUNT", "attestation count changed");

    let draft = named(job(&workflow, "draft-release").unwrap(), "Create draft GitHub release");
    require!(draft.is_some() && enabled(draft.unwrap()) && field(draft.unwrap(), "uses").and_then(Yaml::as_str) == Some("softprops/action-gh-release@3d0d9888cb7fd7b750713d6e236d1fcb99157228") && with_str(draft.unwrap(), "tag_name") == Some("${{ needs.release-identity.outputs.tag }}") && with_str(draft.unwrap(), "target_commitish") == Some("${{ needs.release-identity.outputs.commit }}") && field(field(draft.unwrap(), "with").unwrap(), "draft").and_then(Yaml::as_bool) == Some(true) && with_str(draft.unwrap(), "files").is_some_and(|v| v.contains("lumen-*.tar.gz") && v.contains("lumen-*.tar.gz.sha256")), "INITIAL_DRAFT", "initial release identity/assets/draft state changed");
    let publish = job(&workflow, "publish-release").unwrap();
    let concurrency = field(publish, "concurrency").and_then(Yaml::as_mapping);
    require!(concurrency.is_some() && concurrency.unwrap().get(&key("group")).and_then(Yaml::as_str) == Some("lumen-release-publication") && concurrency.unwrap().get(&key("cancel-in-progress")).and_then(Yaml::as_bool) == Some(false), "PUBLICATION_GUARD", "publication concurrency lock changed");
    let order: Vec<_> = steps(publish).iter().map(|s| field(s, "name").and_then(Yaml::as_str).unwrap_or("")).collect();
    let position = |name| order.iter().position(|v| *v == name).unwrap_or(usize::MAX);
    require!(position("Reconfirm release publication eligibility") < position("Upload SBOM assets to draft release") && position("Upload SBOM assets to draft release") < position("Promote verified root digest to semver and latest tags") && position("Promote verified root digest to semver and latest tags") < position("Finalize release notes and publish release"), "PUBLISH_ORDER", "publish order changed");
    let eligibility = named(publish, "Reconfirm release publication eligibility");
    require!(eligibility.is_some() && enabled(eligibility.unwrap()) && run(eligibility.unwrap()).contains("release is no longer draft") && run(eligibility.unwrap()).contains("git fetch --force --tags origin") && run(eligibility.unwrap()).contains("--sort=-v:refname") && run(eligibility.unwrap()).contains("refusing to move latest from newer tag"), "PUBLICATION_GUARD", "draft/highest-semver publication guard changed");
    let upload = named(publish, "Upload SBOM assets to draft release").map(run).unwrap_or("");
    require!(upload.contains("spdx-amd64.json") && upload.contains("spdx-arm64.json") && upload.contains("--clobber"), "PUBLISH_ORDER", "SBOM draft upload changed");
    let promotion = named(publish, "Promote verified root digest to semver and latest tags").map(run).unwrap_or("");
    require!(promotion.contains("${image_repo}:${semver}") && promotion.contains("${image_repo}:latest") && promotion.matches("${image_repo}@${root_digest}").count() == 2 && promotion.contains("semver_digest=\"$(docker buildx imagetools inspect") && promotion.contains("latest_digest=\"$(docker buildx imagetools inspect") && promotion.contains("\"$semver_digest\" == \"$root_digest\"") && promotion.contains("\"$latest_digest\" == \"$root_digest\""), "PUBLISH_ORDER", "digest promotion or recheck changed");
    for (name, value) in field(&workflow, "jobs").unwrap().as_mapping().unwrap() {
        if name.as_str() != Some("publish-release") { require!(!steps(value).iter().any(|s| run(s).contains("imagetools create")), "PUBLISH_ORDER", "promotion moved before gates"); }
    }
    let finalize = named(publish, "Finalize release notes and publish release").map(run).unwrap_or("");
    for needle in ["- Source commit:", "- Pull request:", "- Workflow run:", "- Root index digest:", "- linux/amd64 digest:", "- linux/arm64 digest:", "--release-state published", "--draft=false"] {
        require!(finalize.contains(needle), "RELEASE_NOTES", format!("release note field missing: {needle}"));
    }
    for name in ["Upload SBOM assets to draft release", "Promote verified root digest to semver and latest tags", "Finalize release notes and publish release"] { require!(named(publish, name).is_some_and(enabled), "PUBLISH_ORDER", format!("publish step disabled: {name}")); }
    let verify_node = named(job(&workflow, "verify-artifacts").unwrap(), "Verify release artifacts against draft release");
    require!(verify_node.is_some_and(enabled) && !run(verify_node.unwrap()).contains("|| true"), "VERIFY_GATE", "draft verifier step is disabled or ignored");
    let verify_step = verify_node.map(run).unwrap_or("");
    for needle in ["apps/lumen/scripts/verify-release-artifacts.sh", "--repo chrischeng-c4/axiom", "--tag \"${{ needs.release-identity.outputs.tag }}\"", "--commit \"${{ needs.release-identity.outputs.commit }}\"", "--image \"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.root_digest }}\"", "--release-state draft"] { require!(verify_step.contains(needle), "VERIFY_GATE", format!("draft verifier invocation missing: {needle}")); }
    for (job_name, runner, machine, binary, checksum, child) in [("kind-amd64", "ubuntu-latest", "x86_64", "kind-linux-amd64", "50030de23cf40a18505f20426f6a8506bedf13c6e509244bd1fa9463721b0f54", "amd64_digest"), ("kind-arm64", "ubuntu-24.04-arm", "aarch64", "kind-linux-arm64", "b92cd615e97585de8ddade28ed5cd7feb4248d717c233eea5b03c37298900f5d", "arm64_digest")] {
        let kind_job = job(&workflow, job_name).unwrap(); let all_runs = steps(kind_job).iter().map(|s| run(s)).collect::<Vec<_>>().join("\n"); let kind_step = named(kind_job, &format!("Run kind e2e in prebuilt mode ({})", if machine == "x86_64" { "amd64" } else { "arm64" }));
        require!(kind_step.is_some_and(enabled) && !run(kind_step.unwrap()).contains("|| true") && field(kind_job, "runs-on").and_then(Yaml::as_str) == Some(runner) && all_runs.contains(&format!("test \"$(uname -m)\" = \"{machine}\"")) && all_runs.contains(&format!("kind/releases/download/v0.32.0/{binary}")) && all_runs.contains(checksum) && all_runs.contains("LUMEN_E2E_IMAGE_MODE=prebuilt") && all_runs.contains("outputs.root_digest") && all_runs.contains(&format!("outputs.{child}")), "KIND_WORKFLOW", format!("{job_name} native digest proof changed"));
    }

    let verifier = &input.verifier;
    let network = verifier.find("release_json=\"$(gh release view").unwrap_or(0);
    let preflight = &verifier[..network];
    for needle in ["REPO\" != \"chrischeng-c4/axiom", "^lumen@[0-9]+\\.[0-9]+\\.[0-9]+$", "^[0-9a-f]{40}$", "^ghcr\\.io/chrischeng-c4/lumen@sha256:[0-9a-f]{64}$", "draft\" && \"$RELEASE_STATE\" != \"published"] {
        require!(preflight.contains(needle), "VERIFIER_PREFLIGHT", format!("preflight missing: {needle}"));
    }
    require!(verifier.contains("source_ref=\"refs/tags/${TAG}\"") && verifier.contains("expected_cert_id=\"https://github.com/${REPO}/.github/workflows/lumen-release.yml@${source_ref}\"") && !verifier.contains("identity-regexp"), "VERIFIER_IDENTITY", "verifier identity is not exact tag identity");
    let attest_fn = shell_fn(verifier, "verify_attestation");
    for flag in ["gh attestation verify \"oci://${subject}\"", "--bundle-from-oci", "--repo \"$REPO\"", "--signer-workflow \"$expected_signer_workflow\"", "--source-ref \"$source_ref\"", "--source-digest \"$COMMIT\"", "--cert-identity \"$expected_cert_id\"", "--cert-oidc-issuer \"$expected_issuer\"", "--predicate-type \"$predicate\"", "--format json"] {
        require!(attest_fn.contains(flag), "VERIFIER_FLAGS", format!("attestation flag missing: {flag}"));
    }
    for needle in ["type == \"array\" and length > 0", "subject | type == \"array\" and length > 0", "all(.verificationResult.statement.subject[];", ".digest.sha256 == $digest"] {
        require!(attest_fn.contains(needle), "VERIFIER_SUBJECT", format!("result enforcement missing: {needle}"));
    }
    for call in ["\"root provenance\" \\\n  \"$IMAGE\" \\\n  \"$root_digest\" \\\n  \"https://slsa.dev/provenance/v1\"", "\"linux/amd64 SBOM\" \\\n  \"${image_repo}@${amd64_digest}\" \\\n  \"$amd64_digest\" \\\n  \"https://spdx.dev/Document/v2.3\"", "\"linux/arm64 SBOM\" \\\n  \"${image_repo}@${arm64_digest}\" \\\n  \"$arm64_digest\" \\\n  \"https://spdx.dev/Document/v2.3\""] {
        require!(verifier.contains(call), "VERIFIER_SUBJECT", "attestation call binding changed");
    }
    for needle in ["gh release view \"$TAG\" --repo \"$REPO\" --json isDraft,tagName", "gh api \"repos/${REPO}/commits/${TAG}\"", "\"$tag_commit\" == \"$COMMIT\"", "--certificate-identity \"$expected_cert_id\"", "--certificate-oidc-issuer \"$expected_issuer\"", ".manifests | type == \"array\" and length == 2", "(.digest | type == \"string\")", "sort == [\"amd64\", \"arm64\"]", "digests must be pairwise distinct"] {
        require!(verifier.contains(needle), "VERIFIER_IDENTITY", format!("verifier proof missing: {needle}"));
    }

    let prebuilt = between(&input.kind, "if [[ \"$IMAGE_MODE\" == \"prebuilt\" ]]; then", "elif [[ \"$IMAGE_MODE\" != \"local\" ]]");
    for needle in ["requires LUMEN_E2E_MODE=operator", "^ghcr\\.io/chrischeng-c4/lumen@sha256:[0-9a-f]{64}$", "^sha256:[0-9a-f]{64}$", "^[0-9a-f]{8}$", "EXPECTED_RUNTIME_DIGEST\" != \"$ROOT_DIGEST", "cargo_ver=\"$(grep"] {
        require!(prebuilt.contains(needle), "KIND_INPUTS", format!("prebuilt input proof missing: {needle}"));
    }
    require!(!["docker build", "kind load docker-image", "docker login", "gh auth"].iter().any(|v| prebuilt.contains(v)), "KIND_PREBUILT_LOCAL", "prebuilt branch performs local or credential work");
    let loader = shell_fn(&input.kind, "build_and_load_image");
    let skipped = loader.find("return 0");
    let build = loader.rfind("\n  docker build -f");
    let load = loader.rfind("\n  kind load docker-image");
    require!(skipped.zip(build).is_some_and(|(a, b)| a < b) && skipped.zip(load).is_some_and(|(a, b)| a < b), "KIND_PREBUILT_LOCAL", "prebuilt does not return before build/load");
    let deploy = shell_fn(&input.kind, "deploy_via_operator");
    for needle in ["kubectl kustomize", "changed != 1", "old_images != 1", "mutable Lumen image remains", "kubectl apply -f \"$tmp_pinned\"", "spec:\n  image: ${IMAGE_TAG}"] {
        require!(deploy.contains(needle), "KIND_PINNING", format!("operator pinning proof missing: {needle}"));
    }
    let normalize = shell_fn(&input.kind, "normalize_runtime_image_id");
    require!(normalize.matches("://").count() == 2 && normalize.contains("docker-pullable://ghcr\\.io/chrischeng-c4/lumen@") && normalize.contains("(containerd|cri-o|docker)://") && normalize.matches("elif [[").count() == 1, "KIND_RUNTIME_ID", "runtime imageID allowlist changed");
    let identity_fn = shell_fn(&input.kind, "assert_cluster_identity");
    for needle in ["deploy/lumen-operator -o json", ".name == \"operator\" and .image == $image", "get lumen/", ".spec.image", "get statefulset/", ".name == \"server\" and .image == $image", "assert_named_pods \"$OPERATOR_NS\"", "assert_named_pods \"$NAMESPACE\"", "/version", "(.version | type) == \"string\"", ".version != \"unknown\""] {
        require!(identity_fn.contains(needle), "KIND_DESIRED_STATE", format!("identity surface missing: {needle}"));
    }
    let first = input.kind.find("step \"4a2. assert cluster identity and /version\" assert_cluster_identity");
    let mutation = input.kind.find("step \"4b. PUT /collections/users\" api_put_collection");
    require!(first.zip(mutation).is_some_and(|(a, b)| a < b), "KIND_ORDER", "identity must precede first API mutation");
    let second = input.kind.find("step \"6c. assert cluster identity and /version post-recovery\" assert_cluster_identity");
    let fresh = input.kind.find("step \"7a. PUT /collections/users after restart\" api_put_collection");
    require!(second.zip(fresh).is_some_and(|(a, b)| a < b), "KIND_POST_RESTART", "post-restart identity must precede fresh write");

    for needle in ["{{json .Manifest}}' | jq -er '.digest'", "[[ \"$RAW_DIGEST\" =~ ^sha256:[0-9a-f]{64}$ ]]", "IMAGE=\"ghcr.io/chrischeng-c4/lumen@${RAW_DIGEST}\"", "--release-state published", "discovery-only", "native amd64 and arm64 kind runs before publication"] {
        require!(input.docs.contains(needle), "DEPLOYMENT_DIGEST", format!("deployment proof missing: {needle}"));
    }
    let verify_docs = between(&input.docs, "Verify release signatures and supply chain attestations before deployment:", "Each release image carries");
    require!(verify_docs.contains("--image \"$IMAGE\""), "DEPLOYMENT_DIGEST", "published verifier must use the retained digest");
    let fetch = between(&input.dockerfile, "FROM debian:bookworm-slim AS binary-source-fetch", "FROM debian:bookworm-slim AS binary-source-staged");
    let staged = between(&input.dockerfile, "FROM debian:bookworm-slim AS binary-source-staged", "FROM binary-source-${SOURCE} AS binary-source");
    for needle in ["ARG SOURCE=fetch", "FROM debian:bookworm-slim AS seed", "FROM binary-source-${SOURCE} AS binary-source", "gcr.io/distroless/static-debian12:nonroot", "ENV LUMEN_HOST=0.0.0.0", "ENTRYPOINT [\"/usr/local/bin/lumen\"]", "CMD [\"serve\"]"] {
        require!(input.dockerfile.contains(needle), "DOCKERFILE_CONTRACT", format!("Dockerfile release contract missing: {needle}"));
    }
    require!(fetch.contains("releases/download/${LUMEN_VERSION}") && fetch.contains("sha256sum -c \"${asset}.sha256\"") && !["curl", "apt-get", "releases/download"].iter().any(|v| staged.contains(v)), "DOCKERFILE_CONTRACT", "fetch/staged sources are not isolated");
    let cargo: toml::Value = toml::from_str(&input.cargo).map_err(|e| Finding { code: "TOML_PARSE", detail: e.to_string() })?;
    let registered = cargo.get("test").and_then(toml::Value::as_array).map_or(0, |tests| tests.iter().filter(|t| t.get("name").and_then(toml::Value::as_str) == Some("release_artifacts") && t.get("path").and_then(toml::Value::as_str) == Some("e2e/release_artifacts.rs")).count());
    require!(registered == 1, "CARGO_REGISTRATION", "release oracle must be registered exactly once");
    require!(input.verifier_mode & 0o777 == 0o755, "VERIFIER_MODE", "verifier mode must be 0755");
    for needle in ["ARG SOURCE=fetch", "ARG LUMEN_VERSION=lumen@9.9.9", "releases/download/${LUMEN_VERSION}", "sha256sum -c \"${asset}.sha256\"", "FROM binary-source-${SOURCE}", "--build-arg LUMEN_VERSION=lumen@9.9.9"] {
        require!(input.rendered.contains(needle), "PUBLIC_RENDER", format!("public render missing: {needle}"));
    }
    require!(!input.rendered.contains("SOURCE=staged"), "PUBLIC_RENDER", "public render selected staged source");
    Ok(())
}

#[rustfmt::skip]
fn render_release() -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_lumen")).args(["dockerfile", "render", "--variant", "release", "--version", "9.9.9"]).output().expect("run lumen dockerfile render");
    assert!(output.status.success(), "render failed: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).expect("render is UTF-8")
}
#[rustfmt::skip]
fn live() -> Inputs {
    let root = root();
    let read = |path: &str| fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let verifier = root.join("apps/lumen/scripts/verify-release-artifacts.sh");
    Inputs { workflow: read(".github/workflows/lumen-release.yml"), verifier: read("apps/lumen/scripts/verify-release-artifacts.sh"), kind: read("apps/lumen/scripts/kind-e2e.sh"), docs: read("apps/lumen/docs/deployment.md"), dockerfile: read("apps/lumen/Dockerfile.release"), cargo: read("apps/lumen/Cargo.toml"), verifier_mode: fs::metadata(verifier).unwrap().permissions().mode(), rendered: render_release() }
}
#[rustfmt::skip]
fn replace_once(source: &str, from: &str, to: &str) -> String {
    assert_ne!(from, to, "mutation must change bytes");
    assert_eq!(source.matches(from).count(), 1, "mutation target {from:?} is not unique");
    let changed = source.replacen(from, to, 1); assert_ne!(changed, source); changed
}
#[rustfmt::skip]
fn replace_nth(source: &str, from: &str, to: &str, nth: usize) -> String {
    let offsets: Vec<_> = source.match_indices(from).map(|(at, _)| at).collect();
    let at = *offsets.get(nth).unwrap_or_else(|| panic!("mutation target {from:?} has only {} sites", offsets.len()));
    let changed = format!("{}{}{}", &source[..at], to, &source[at + from.len()..]); assert_ne!(changed, source); changed
}
fn job_replace(source: &str, name: &str, from: &str, to: &str) -> String {
    let marker = format!("\n  {name}:\n");
    let start = source.find(&marker).expect("job exists") + 1;
    let tail = &source[start + marker.len() - 1..];
    let end = tail
        .match_indices("\n  ")
        .find_map(|(i, _)| {
            tail[i + 3..]
                .chars()
                .next()
                .is_some_and(|c| !c.is_whitespace())
                .then_some(start + marker.len() - 1 + i)
        })
        .unwrap_or(source.len());
    let changed = replace_once(&source[start..end], from, to);
    format!("{}{}{}", &source[..start], changed, &source[end..])
}
fn function_replace(source: &str, name: &str, from: &str, to: &str) -> String {
    let marker = format!("{name}() {{");
    let start = source.find(&marker).expect("function exists");
    let tail = &source[start..];
    let end = start + tail.find("\n}\n").expect("function closes") + 3;
    let changed = replace_once(&source[start..end], from, to);
    format!("{}{}{}", &source[..start], changed, &source[end..])
}
fn expect(mutated: Inputs, code: &'static str) {
    let finding = validate(&mutated).expect_err("negative mutation passed");
    assert_eq!(finding.code, code, "wrong finding: {finding:?}");
}

#[test]
fn live_release_artifacts_satisfy_contract() {
    validate(&live()).expect("live release artifacts satisfy the frozen contract");
}

#[test]
#[rustfmt::skip]
fn scoped_negative_mutations_fail_with_stable_findings() {
    let cases = [
        ("ghcr-image-and-attest", "name: Sign root image index with keyless cosign\n        shell:", "name: Sign root image index with keyless cosign\n        if: false\n        shell:", "ROOT_SIGNATURE"),
        ("ghcr-image-and-attest", "      id-token: write\n", "", "IMAGE_PERMISSIONS"),
        ("ghcr-image-and-attest", "      attestations: write\n", "", "IMAGE_PERMISSIONS"),
        ("ghcr-image-and-attest", "      contents: read\n", "      contents: write\n", "IMAGE_PERMISSIONS"),
        ("ghcr-image-and-attest", "subject-digest: ${{ steps.push.outputs.digest }}", "subject-digest: ${{ steps.platform_digests.outputs.amd64_digest }}", "ROOT_PROVENANCE"),
        ("ghcr-image-and-attest", "name: Generate SPDX SBOM for linux/arm64\n        uses:", "name: Generate SPDX SBOM for linux/arm64\n        if: false\n        uses:", "PLATFORM_SBOM"),
        ("ghcr-image-and-attest", "subject-digest: ${{ steps.platform_digests.outputs.arm64_digest }}", "subject-digest: ${{ steps.platform_digests.outputs.amd64_digest }}", "PLATFORM_SBOM"),
        ("ghcr-image-and-attest", "platforms: linux/amd64,linux/arm64", "platforms: linux/amd64,linux/arm64,linux/s390x", "IMAGE_PLATFORMS"),
        ("draft-release", "          draft: true", "          draft: false", "INITIAL_DRAFT"),
        ("publish-release", "      packages: write\n", "", "PUBLISH_PERMISSIONS"),
        ("ghcr-image-and-attest", "SOURCE=staged", "SOURCE=fetch", "STAGED_SOURCE"),
        ("ghcr-image-and-attest", "sha256sum -c lumen-x86_64-unknown-linux-musl.tar.gz.sha256", "true", "MUSL_CHECKSUM"),
        ("ghcr-image-and-attest", "sha256sum -c lumen-aarch64-unknown-linux-musl.tar.gz.sha256", "true", "MUSL_CHECKSUM"),
    ];
    for (job, from, to, code) in cases {
        let mut fixture = live(); fixture.workflow = job_replace(&fixture.workflow, job, from, to); expect(fixture, code);
    }
    for (from, to, code) in [("provenance: false", "provenance: true", "BUILDKIT_POLICY"), ("sbom: false", "sbom: true", "BUILDKIT_POLICY")] {
        let mut fixture = live(); fixture.workflow = job_replace(&fixture.workflow, "ghcr-image-and-attest", from, to); expect(fixture, code);
    }
    let mut fixture = live(); fixture.workflow = job_replace(&fixture.workflow, "ghcr-image-and-attest", "subject-digest: ${{ steps.push.outputs.digest }}\n          push-to-registry: true", "subject-digest: ${{ steps.push.outputs.digest }}\n          push-to-registry: false"); expect(fixture, "ROOT_PROVENANCE");
    for identity in ["refs/heads/main", "refs/pull/1/merge"] {
        let mut fixture = live(); fixture.verifier = replace_once(&fixture.verifier, "refs/tags/${TAG}", identity); expect(fixture, "VERIFIER_IDENTITY");
    }
    let mut fixture = live(); fixture.verifier = replace_once(&fixture.verifier, "--certificate-identity \"$expected_cert_id\"", "--certificate-identity-regexp '.*'"); expect(fixture, "VERIFIER_IDENTITY");
    for gate in [", kind-amd64", ", kind-arm64"] {
        let mut fixture = live(); fixture.workflow = job_replace(&fixture.workflow, "publish-release", gate, ""); expect(fixture, "JOB_GRAPH");
    }
    let mut fixture = live(); fixture.workflow = replace_once(&fixture.workflow, "workflow_dispatch:\n", "workflow_dispatch:\n    inputs:\n      image_version:\n        required: false\n"); expect(fixture, "MANUAL_ROUTE");
    for replacement in ["dtolnay/rust-toolchain@stable # stable", "dtolnay/rust-toolchain@0000000000000000000000000000000000000000 # stable"] {
        let mut fixture = live(); fixture.workflow = job_replace(&fixture.workflow, "build", "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c # stable", replacement); expect(fixture, "ACTION_PIN");
    }
    let mut fixture = live(); fixture.workflow = job_replace(&fixture.workflow, "ghcr-image-and-attest", "      - name: Sign root image index with keyless cosign", "      - name: Premature semver promotion\n        shell: bash\n        run: docker buildx imagetools create -t ${{ steps.tags.outputs.image_repo }}:${{ needs.release-identity.outputs.version }} ${{ steps.tags.outputs.image_repo }}@${{ steps.push.outputs.digest }}\n      - name: Sign root image index with keyless cosign"); expect(fixture, "PUBLISH_ORDER");
    let mut fixture = live(); fixture.workflow = job_replace(&fixture.workflow, "publish-release", "group: lumen-release-publication", "group: per-run-publication"); expect(fixture, "PUBLICATION_GUARD");
    let mut fixture = live(); fixture.workflow = job_replace(&fixture.workflow, "verify-artifacts", "name: Verify release artifacts against draft release\n        env:", "name: Verify release artifacts against draft release\n        if: false\n        env:"); expect(fixture, "VERIFY_GATE");
    for (job, arch) in [("kind-amd64", "amd64"), ("kind-arm64", "arm64")] {
        let mut fixture = live(); let marker = format!("name: Run kind e2e in prebuilt mode ({arch})\n        shell:"); let disabled = format!("name: Run kind e2e in prebuilt mode ({arch})\n        if: false\n        shell:"); fixture.workflow = job_replace(&fixture.workflow, job, &marker, &disabled); expect(fixture, "KIND_WORKFLOW");
    }
    let mut fixture = live(); fixture.docs = replace_nth(&fixture.docs, "--image \"$IMAGE\"", "--image ghcr.io/chrischeng-c4/lumen:latest", 0); expect(fixture, "DEPLOYMENT_DIGEST");
    let mut fixture = live(); fixture.kind = replace_once(&fixture.kind, "^ghcr\\.io/chrischeng-c4/lumen@sha256:[0-9a-f]{64}$", "^ghcr\\.io/chrischeng-c4/lumen:.+$"); expect(fixture, "KIND_INPUTS");
    let mut fixture = live(); fixture.kind = replace_nth(&fixture.kind, "if [[ \"$IMAGE_MODE\" == \"prebuilt\" ]]; then\n", "if [[ \"$IMAGE_MODE\" == \"prebuilt\" ]]; then\n  docker login ghcr.io\n", 0); expect(fixture, "KIND_PREBUILT_LOCAL");
    for (from, to) in [
        ("  op_json=\"$(kubectl -n \"$OPERATOR_NS\" get deploy/lumen-operator -o json)\"", "  op_json=\"$(jq -nc --arg image \"$IMAGE_TAG\" '{spec:{replicas:1,template:{spec:{containers:[{name:\"operator\",image:$image}]}}}}')\""),
        ("  cr_img=\"$(kubectl -n \"$NAMESPACE\" get lumen/\"${LUMEN_CR_NAME}\" -o jsonpath='{.spec.image}')\"", "  cr_img=\"$IMAGE_TAG\""),
        ("  sset_json=\"$(kubectl -n \"$NAMESPACE\" get statefulset/\"${LUMEN_CR_NAME}\" -o json)\"", "  sset_json=\"$(jq -nc --arg image \"$IMAGE_TAG\" '{spec:{replicas:1,template:{spec:{containers:[{name:\"server\",image:$image}]}}}}')\""),
        ("  assert_named_pods \"$OPERATOR_NS\" \"app.kubernetes.io/name=lumen-operator\" operator \"$op_replicas\"", "  : # operator Pod identity check omitted"),
        ("  assert_named_pods \"$NAMESPACE\" \"$APP_LABEL\" server \"$sset_replicas\"", "  : # serving Pod identity check omitted"),
    ] {
        let mut fixture = live(); fixture.kind = function_replace(&fixture.kind, "assert_cluster_identity", from, to); expect(fixture, "KIND_DESIRED_STATE");
    }
    let mut fixture = live(); fixture.kind = replace_once(&fixture.kind, "step \"4a2. assert cluster identity and /version\" assert_cluster_identity\nstep \"4b. PUT /collections/users\" api_put_collection", "step \"4b. PUT /collections/users\" api_put_collection\nstep \"4a2. assert cluster identity and /version\" assert_cluster_identity"); expect(fixture, "KIND_ORDER");
    let mut fixture = live(); fixture.kind = replace_once(&fixture.kind, "step \"6c. assert cluster identity and /version post-recovery\" assert_cluster_identity", "echo post-restart-identity-omitted"); expect(fixture, "KIND_POST_RESTART");
    let mut fixture = live(); fixture.verifier = replace_once(&fixture.verifier, "    --source-digest \"$COMMIT\" \\\n", ""); expect(fixture, "VERIFIER_FLAGS");
    let mut fixture = live(); fixture.cargo = replace_once(&fixture.cargo, "name = \"release_artifacts\"", "name = \"release_artifacts_disabled\""); expect(fixture, "CARGO_REGISTRATION");
    let mut fixture = live(); fixture.verifier_mode = 0o644; expect(fixture, "VERIFIER_MODE");
    let mut fixture = live(); fixture.dockerfile = replace_once(&fixture.dockerfile, "ARG SOURCE=fetch", "ARG SOURCE=staged"); expect(fixture, "DOCKERFILE_CONTRACT");
}
