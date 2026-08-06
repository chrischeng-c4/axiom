// HANDWRITE-BEGIN gap="missing-generator:e2e-test:lumen-client-access-render" tracker="#2889" reason="Proving a rendered RBAC bundle keeps two identities apart needs the shipped CLI surface plus a structural read of the emitted objects; no generator emits either."
#![cfg(feature = "operator")]
//! #2889: `lumen k8s access render` renders the two-hop identity handoff.
//!
//! The bundle exists because the one-hop version is easy to reach by
//! accident. Binding the human's Kubernetes username straight to Lumen's
//! collection role authorizes the right person, applies cleanly, and reads
//! like it worked — and then every request is denied, because what arrives at
//! Lumen is a ServiceAccount token nobody granted anything to. So the
//! assertions here are mostly about *which subject is on which binding*, and
//! they are deliberately paired: the issuer binding must name the user and not
//! the ServiceAccount, and the Lumen binding must name the ServiceAccount and
//! not the user. Either assertion alone passes on a bundle that has both
//! subjects on one binding.
//!
//! The live half — that the issuer can actually mint the named token and not a
//! sibling's, and that the rendered collection grant authorizes what it says —
//! is the GKE proof recorded on the issue. It needs an API server; these need
//! only the binary.

use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

/// Run `lumen` with `args`, requiring success.
fn lumen(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("run lumen {args:?}: {err}"));
    assert!(
        output.status.success(),
        "lumen {args:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("lumen stdout is utf8")
}

/// Run `lumen` with `args`, requiring failure, and return stderr.
fn lumen_err(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("run lumen {args:?}: {err}"));
    assert!(
        !output.status.success(),
        "lumen {args:?} unexpectedly succeeded\nstdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    String::from_utf8(output.stderr).expect("lumen stderr is utf8")
}

/// Parse a rendered bundle into its documents, keyed by `<kind>/<name>`.
fn documents(yaml: &str) -> Vec<(String, Value)> {
    serde_yaml::Deserializer::from_str(yaml)
        .map(|document| {
            let value: Value =
                serde_path_to_error::deserialize(document).expect("bundle document is valid YAML");
            let key = format!(
                "{}/{}",
                value["kind"].as_str().expect("document has a kind"),
                value["metadata"]["name"]
                    .as_str()
                    .expect("document has a name")
            );
            (key, value)
        })
        .collect()
}

fn find<'a>(documents: &'a [(String, Value)], key: &str) -> &'a Value {
    &documents
        .iter()
        .find(|(name, _)| name == key)
        .unwrap_or_else(|| {
            panic!(
                "bundle has no `{key}`; it has {:?}",
                documents.iter().map(|(name, _)| name).collect::<Vec<_>>()
            )
        })
        .1
}

/// The bundle every assertion below reads: one client, two issuers, a
/// read-only and a read-write collection, and the admin surface.
fn full_bundle() -> String {
    lumen(&[
        "k8s",
        "access",
        "render",
        "--namespace",
        "search",
        "--client-sa",
        "app-client",
        "--issuer",
        "alice@example.com",
        "--issuer",
        "lumen-client@proj.iam.gserviceaccount.com",
        "--grant",
        "docs=read",
        "--grant",
        "orders=write",
        "--instance-admin",
    ])
}

/// AC1: the issuer Role is exactly one rule, and that rule names the one
/// ServiceAccount it may mint a token for.
///
/// `create` on `serviceaccounts/token` without `resourceNames` is a namespace
/// takeover: it mints a token for every ServiceAccount there, including the
/// operator's. The assertion is on the whole rule rather than on its fields
/// one at a time, so a rule that grew a second resource, a second verb, or a
/// second name fails here rather than in a cluster.
#[test]
fn the_issuer_role_grants_exactly_one_verb_on_one_named_service_account() {
    let documents = documents(&full_bundle());
    let role = find(&documents, "Role/app-client-token-issuer");
    assert_eq!(role["metadata"]["namespace"], "search");
    assert_eq!(
        role["rules"],
        serde_json::json!([{
            "apiGroups": [""],
            "resources": ["serviceaccounts/token"],
            "resourceNames": ["app-client"],
            "verbs": ["create"],
        }]),
        "the issuer rule must stay one verb on one named ServiceAccount"
    );
}

/// AC3 (render half): a `read` grant carries `get` and nothing else, and the
/// collections it does not name appear nowhere in the Role.
///
/// The live denial half — `kubectl auth can-i` against a real API server —
/// is the GKE proof; this pins what was asked for, so a rendered grant that
/// quietly widened is caught without a cluster.
#[test]
fn a_read_grant_asks_for_the_read_verb_and_no_other() {
    let documents = documents(&full_bundle());
    let role = find(&documents, "Role/app-client-lumen-access");
    let rules = role["rules"].as_array().expect("the Lumen role has rules");

    let docs = rules
        .iter()
        .find(|rule| rule["resourceNames"] == serde_json::json!(["docs"]))
        .expect("the bundle has a rule for the `docs` collection");
    assert_eq!(docs["apiGroups"], serde_json::json!(["lumen.axiom.dev"]));
    assert_eq!(docs["resources"], serde_json::json!(["lumencollections"]));
    assert_eq!(
        docs["verbs"],
        serde_json::json!(["get"]),
        "a read grant that carries `update` or `delete` is not read-only"
    );

    assert!(
        !rules
            .iter()
            .any(|rule| rule["resourceNames"] == serde_json::json!(["invoices"])),
        "the bundle names a collection nobody granted: {rules:?}"
    );
}

/// A `write` grant carries `get` too: the mapping from role to verb is
/// one-to-one, but the grant is cumulative, because a client that could write
/// a collection and not read it would be denied by the same check that let it
/// write. `admin` on a collection carries all three.
#[test]
fn a_grant_carries_every_verb_at_or_below_its_level() {
    let bundle = documents(&full_bundle());
    let rules = find(&bundle, "Role/app-client-lumen-access")["rules"]
        .as_array()
        .expect("the Lumen role has rules")
        .clone();
    let orders = rules
        .iter()
        .find(|rule| rule["resourceNames"] == serde_json::json!(["orders"]))
        .expect("the bundle has a rule for the `orders` collection");
    assert_eq!(orders["verbs"], serde_json::json!(["get", "update"]));

    let admin_collection = documents(&lumen(&[
        "k8s",
        "access",
        "render",
        "--namespace",
        "search",
        "--client-sa",
        "app-client",
        "--issuer",
        "alice@example.com",
        "--grant",
        "orders=admin",
    ]));
    assert_eq!(
        find(&admin_collection, "Role/app-client-lumen-access")["rules"][0]["verbs"],
        serde_json::json!(["get", "update", "delete"])
    );
}

/// `--instance-admin` grants the instance-wide surface, and only it. There is
/// no `resourceNames` because `AuthTarget::Admin` sends no resource name, and
/// only `delete` because that is the single role every `ensure_admin` call
/// site asks for — see the guard below.
#[test]
fn the_instance_admin_grant_is_the_admin_resource_at_one_verb() {
    let bundle = documents(&full_bundle());
    let rules = find(&bundle, "Role/app-client-lumen-access")["rules"]
        .as_array()
        .expect("the Lumen role has rules")
        .clone();
    let admin = rules
        .iter()
        .find(|rule| rule["resources"] == serde_json::json!(["lumenadmin"]))
        .expect("--instance-admin renders a lumenadmin rule");
    assert_eq!(admin["apiGroups"], serde_json::json!(["lumen.axiom.dev"]));
    assert_eq!(admin["verbs"], serde_json::json!(["delete"]));
    assert!(
        admin.get("resourceNames").is_none(),
        "an empty resourceNames list matches no object at all: {admin:?}"
    );

    let without = documents(&lumen(&[
        "k8s",
        "access",
        "render",
        "--namespace",
        "search",
        "--client-sa",
        "app-client",
        "--issuer",
        "alice@example.com",
        "--grant",
        "docs=read",
    ]));
    let rules = find(&without, "Role/app-client-lumen-access")["rules"]
        .as_array()
        .expect("the Lumen role has rules")
        .clone();
    assert!(
        !rules
            .iter()
            .any(|rule| rule["resources"] == serde_json::json!(["lumenadmin"])),
        "the admin surface must be opt-in: {rules:?}"
    );
}

/// The verb the admin rule carries is derived from one fact about the serving
/// side: every admin endpoint checks `Role::Admin`. If an endpoint ever
/// checked a lower role, `delete` alone would deny it and the failure would
/// look like a misconfigured cluster. Pin the fact, not the consequence.
#[test]
fn every_admin_endpoint_still_checks_the_admin_role() {
    let api = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("api.rs"),
    )
    .expect("read src/api.rs");
    let call = "ensure_admin(";
    let mut sites = 0;
    let mut from = 0;
    while let Some(offset) = api[from..].find(call) {
        let start = from + offset + call.len();
        assert!(
            api[start..].starts_with("Role::Admin)"),
            "src/api.rs checks the admin surface at a role other than `Role::Admin`; \
             `--instance-admin` renders only the verb `Role::Admin` maps to, so that \
             endpoint would be denied:\n{}",
            &api[start..start + 40.min(api.len() - start)]
        );
        sites += 1;
        from = start;
    }
    assert!(
        sites >= 8,
        "expected the admin surface to have several endpoints, found {sites}"
    );
}

/// AC4: the two bindings carry different subject kinds, and neither carries
/// the other's.
///
/// This is the whole boundary. The issuer binding names people, who
/// authenticate to kube-apiserver; the Lumen binding names the ServiceAccount,
/// which is the only identity Lumen ever sees. A bundle that put the Google
/// principal on the Lumen binding would authorize a caller who never arrives.
#[test]
fn the_two_bindings_never_swap_their_subjects() {
    let documents = documents(&full_bundle());

    let issuer = find(&documents, "RoleBinding/app-client-token-issuer");
    assert_eq!(issuer["roleRef"]["kind"], "Role");
    assert_eq!(issuer["roleRef"]["name"], "app-client-token-issuer");
    assert_eq!(
        issuer["subjects"],
        serde_json::json!([
            {
                "apiGroup": "rbac.authorization.k8s.io",
                "kind": "User",
                "name": "alice@example.com",
            },
            {
                "apiGroup": "rbac.authorization.k8s.io",
                "kind": "User",
                "name": "lumen-client@proj.iam.gserviceaccount.com",
            },
        ]),
        "the issuer binding names the external users and nothing else"
    );

    let access = find(&documents, "RoleBinding/app-client-lumen-access");
    assert_eq!(access["roleRef"]["name"], "app-client-lumen-access");
    assert_eq!(
        access["subjects"],
        serde_json::json!([
            { "kind": "ServiceAccount", "name": "app-client", "namespace": "search" },
        ]),
        "the Lumen binding names the client ServiceAccount and nothing else"
    );

    // The serving ServiceAccount is not the client ServiceAccount. Binding the
    // issuer role to it would let a caller mint the fleet's own identity.
    let rendered = full_bundle();
    assert!(
        !rendered.contains("name: search\n"),
        "the bundle names the serving ServiceAccount:\n{rendered}"
    );
}

/// R2: the bundle declares the ServiceAccount it grants access to, so applying
/// it is one step and the RoleBinding cannot dangle on a name that does not
/// exist.
#[test]
fn the_bundle_creates_the_client_service_account() {
    let documents = documents(&full_bundle());
    let sa = find(&documents, "ServiceAccount/app-client");
    assert_eq!(sa["apiVersion"], "v1");
    assert_eq!(sa["metadata"]["namespace"], "search");
    assert!(
        sa.get("secrets").is_none() && sa.get("imagePullSecrets").is_none(),
        "the ServiceAccount references a Secret: {sa:?}"
    );
}

/// AC5: nothing in the bundle is a credential, and nothing in it is a
/// wildcard.
///
/// The needles are assembled rather than written so this file does not itself
/// contain the strings it forbids — a gate that matches its own source is one
/// nobody can keep green.
#[test]
fn no_rendered_object_carries_a_credential_or_a_wildcard() {
    let rendered = full_bundle();

    let forbidden: &[(String, &str)] = &[
        (
            ["Secret"].concat(),
            "a Secret is a durable credential; this bundle mints short-lived tokens instead",
        ),
        (
            ["secret", "KeyRef"].concat(),
            "an env credential reference has no place in an RBAC grant",
        ),
        (
            ["Bearer "].concat(),
            "a rendered manifest must never carry a presented token",
        ),
        (
            ["ey", "J"].concat(),
            "a JWT prefix in a rendered manifest means a token was embedded",
        ),
        (
            ["accounts.", "google.com"].concat(),
            "a Google OAuth audience is not part of the Kubernetes handoff",
        ),
        (
            ["iam.", "gserviceaccount.com/roles"].concat(),
            "a GCP IAM role is not a Kubernetes grant",
        ),
        (
            ["roles/", "iam."].concat(),
            "a GCP IAM role is not a Kubernetes grant",
        ),
        (
            ["workload", "IdentityPool"].concat(),
            "Workload Identity Federation is out of this bundle's scope",
        ),
    ];
    for (needle, why) in forbidden {
        assert!(
            !rendered.contains(needle.as_str()),
            "the rendered bundle contains `{needle}` — {why}:\n{rendered}"
        );
    }

    // A `*` anywhere: `verbs: ["*"]`, `resources: ["pods/*"]`, and a subject
    // named `*` are three different mistakes and one character.
    assert!(
        !rendered.contains('*'),
        "the rendered bundle contains a wildcard:\n{rendered}"
    );
}

/// AC6: stdout is the artifact. No envelope, no `next:` line, nothing a
/// `kubectl apply -f -` would choke on.
#[test]
fn stdout_is_raw_multi_document_yaml() {
    let rendered = full_bundle();
    assert!(
        !rendered.contains("next:"),
        "a `next:` line on stdout would be applied as YAML:\n{rendered}"
    );
    let documents = documents(&rendered);
    assert_eq!(
        documents
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        [
            "ServiceAccount/app-client",
            "Role/app-client-token-issuer",
            "RoleBinding/app-client-token-issuer",
            "Role/app-client-lumen-access",
            "RoleBinding/app-client-lumen-access",
        ],
        "the bundle is five objects in apply order"
    );
}

/// AC6: `--out` writes the same bytes and emits exactly one runnable follow-up
/// — the artifact convention every other `lumen k8s ... render` verb keeps.
#[test]
fn out_writes_the_bundle_and_names_the_command_that_applies_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("bundle.yaml");
    let stdout = lumen(&[
        "k8s",
        "access",
        "render",
        "--namespace",
        "search",
        "--client-sa",
        "app-client",
        "--issuer",
        "alice@example.com",
        "--grant",
        "docs=read",
        "--out",
        path.to_str().expect("utf8 path"),
    ]);
    assert_eq!(
        stdout.trim(),
        format!(
            "wrote {path}\nnext: kubectl apply -f {path}",
            path = path.display()
        ),
        "the file-writing branch reports the path and one runnable next step, \
         and no artifact bytes"
    );

    let written = std::fs::read_to_string(&path).expect("read the written bundle");
    let streamed = lumen(&[
        "k8s",
        "access",
        "render",
        "--namespace",
        "search",
        "--client-sa",
        "app-client",
        "--issuer",
        "alice@example.com",
        "--grant",
        "docs=read",
    ]);
    assert_eq!(
        written, streamed,
        "`--out` and stdout must agree byte for byte"
    );

    // A directory receives the conventional file name.
    let stdout = lumen(&[
        "k8s",
        "access",
        "render",
        "--namespace",
        "search",
        "--client-sa",
        "app-client",
        "--issuer",
        "alice@example.com",
        "--grant",
        "docs=read",
        "--out",
        dir.path().to_str().expect("utf8 path"),
    ]);
    assert!(
        stdout.trim().ends_with("access.yaml"),
        "a directory target receives `access.yaml`: {stdout}"
    );
}

/// AC6: a malformed grant is rejected by name, with the spelling that would
/// have worked. Every one of these is a plausible typo, and every one of them
/// would otherwise render a grant that is valid RBAC and wrong.
#[test]
fn a_malformed_grant_is_refused_with_the_shape_it_should_have_had() {
    let base = [
        "k8s",
        "access",
        "render",
        "--namespace",
        "search",
        "--client-sa",
        "app-client",
        "--issuer",
        "alice@example.com",
    ];
    let cases: &[(&[&str], &str)] = &[
        (&["--grant", "docs"], "read|write|admin"),
        (
            &["--grant", "docs=readonly"],
            "must be `read`, `write`, or `admin`",
        ),
        (&["--grant", "=read"], "collection id"),
        (&["--grant", "do cs=read"], "no whitespace"),
        (&["--grant", "*=read"], "no `*`"),
        (&["--grant", "docs=read", "--grant", "docs=admin"], "twice"),
        (&[], "at least one"),
    ];
    for (extra, expected) in cases {
        let mut args = base.to_vec();
        args.extend_from_slice(extra);
        let stderr = lumen_err(&args);
        assert!(
            stderr.contains(expected),
            "`{extra:?}` should be refused with a message naming `{expected}`, got:\n{stderr}"
        );
    }
}

/// AC6: the names the bundle is built from are checked too. A namespace or
/// ServiceAccount name that is not a DNS label renders objects the API server
/// rejects; a wildcard issuer renders a binding that matches everyone.
#[test]
fn a_malformed_name_is_refused_before_anything_is_rendered() {
    let cases: &[(&[&str], &str)] = &[
        (
            &[
                "--namespace",
                "Search",
                "--client-sa",
                "app-client",
                "--issuer",
                "alice@example.com",
                "--grant",
                "docs=read",
            ],
            "--namespace must be a DNS-1123 label",
        ),
        (
            &[
                "--namespace",
                "search",
                "--client-sa",
                "app_client",
                "--issuer",
                "alice@example.com",
                "--grant",
                "docs=read",
            ],
            "--client-sa must be a DNS-1123 label",
        ),
        (
            &[
                "--namespace",
                "search",
                "--client-sa",
                "app-client",
                "--issuer",
                "*",
                "--grant",
                "docs=read",
            ],
            "--issuer must be the username",
        ),
        (
            &[
                "--namespace",
                "search",
                "--client-sa",
                "app-client",
                "--issuer",
                " alice@example.com",
                "--grant",
                "docs=read",
            ],
            "no surrounding whitespace",
        ),
    ];
    for (extra, expected) in cases {
        let mut args = vec!["k8s", "access", "render"];
        args.extend_from_slice(extra);
        let stderr = lumen_err(&args);
        assert!(
            stderr.contains(expected),
            "`{extra:?}` should be refused with `{expected}`, got:\n{stderr}"
        );
    }

    // `--issuer` is required: a bundle with no issuer would create a
    // ServiceAccount nobody can get a token for.
    let stderr = lumen_err(&[
        "k8s",
        "access",
        "render",
        "--namespace",
        "search",
        "--client-sa",
        "app-client",
        "--grant",
        "docs=read",
    ]);
    assert!(
        stderr.contains("--issuer"),
        "the issuer list is required:\n{stderr}"
    );
}

/// AC6: the help page teaches the boundary and advertises no credential flag.
#[test]
fn the_help_page_names_the_boundary_and_no_credential_flag() {
    let help = lumen(&["k8s", "access", "render", "--help"]);
    for flag in [
        "--namespace",
        "--client-sa",
        "--issuer",
        "--grant",
        "--instance-admin",
        "--out",
    ] {
        assert!(help.contains(flag), "`{flag}` is missing from:\n{help}");
    }
    assert!(
        help.contains("kubectl auth whoami"),
        "the help page should say where an issuer name comes from:\n{help}"
    );
    for forbidden in [
        ["--", "token"].concat(),
        ["--", "credential"].concat(),
        ["--", "secret"].concat(),
        ["--", "key-file"].concat(),
    ] {
        assert!(
            !help.contains(&forbidden),
            "`lumen k8s access render` advertises `{forbidden}`; this command \
             renders names, never material:\n{help}"
        );
    }

    // R1: render-only. Nothing here shells out or applies.
    let group = lumen(&["k8s", "access", "--help"]);
    assert!(
        group.contains("render"),
        "the access group should expose `render`:\n{group}"
    );
    for verb in ["apply", "grant ", "create "] {
        assert!(
            !group.contains(verb),
            "`lumen k8s access` exposes a mutating verb `{verb}`:\n{group}"
        );
    }
}
// HANDWRITE-END
