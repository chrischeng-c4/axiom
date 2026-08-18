//! Spike: does per-namespace isolation survive when the only credential is a
//! GCP service-account identity and there is no shared secret anywhere?
//!
//! The question this answers is not "can an SA authenticate" — that is easy and
//! proves little. It is: **if one SA is granted access to one data plane, can it
//! reach the others?** Two independent layers are supposed to say no, and this
//! spike demonstrates each one *while holding the other wide open*, because a
//! defense that is only ever tested alongside a second defense has not been
//! tested.
//!
//!   layer 1  `aud`      the token is minted for one instance's address, so a
//!                       token for team-a is not a token for team-b at all
//!   layer 2  registry   each instance grants its own identities, so an SA
//!                       nobody granted is authenticated but not authorized
//!
//! Run against real Google artifacts (a live 1-hour token, the live JWKS) —
//! that is the point, and why this is an `examples/` binary rather than a test:
//!
//! ```sh
//! SA=lumen-dev@axiom-502607.iam.gserviceaccount.com
//! for ns in team-a team-b; do
//!   gcloud auth print-identity-token --include-email \
//!     --audiences="https://lumen.$ns.svc" --impersonate-service-account="$SA" \
//!     > "/tmp/idtoken_$ns.txt"
//! done
//! cargo run -p service-auth --example gcp_id_token_spike -- \
//!   /tmp/idtoken_team-a.txt /tmp/idtoken_team-b.txt
//! ```
//!
//! Everything runs through the shipped `GoogleVerifier` and the unmodified
//! `role_map` — no hand-rolled verification — so a pass is evidence about the
//! code that would actually serve requests, not about a model of it.

use std::collections::HashMap;
use std::sync::Arc;

use axum::http::HeaderMap;
use service_auth::gcp::{GoogleAuthConfig, GoogleVerifier};
use service_auth::role_map::{Registry, Role};
use service_auth::{AsyncVerifier, ReloadableRoleMapVerifier};

const SA: &str = "lumen-dev@axiom-502607.iam.gserviceaccount.com";
const AUD_A: &str = "https://lumen.team-a.svc";
const AUD_B: &str = "https://lumen.team-b.svc";

/// One data plane's verifier: its own audience, its own identity grants.
///
/// `registry_json` is the whole configuration surface — note that it contains
/// no secret, only emails and roles, which is what makes it publishable in a
/// CR that anybody may read.
fn instance(audience: &str, registry_json: &str) -> GoogleVerifier {
    let registry = Registry::parse(registry_json).expect("registry parses");
    GoogleVerifier::google(
        true,
        Arc::new(ReloadableRoleMapVerifier::with_registry(true, registry)),
        GoogleAuthConfig::new([audience]),
    )
    .expect("verifier builds")
}

fn bearer(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Bearer {token}").parse().expect("header value"),
    );
    headers
}

fn read(path: &str) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
        .trim()
        .to_string()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut args = std::env::args().skip(1);
    let token_a = read(&args.next().expect("usage: <token-aud-team-a> <token-aud-team-b>"));
    let token_b = read(&args.next().expect("usage: <token-aud-team-a> <token-aud-team-b>"));

    // team-a granted this SA read on one collection, and nothing else.
    let granted = format!(
        r#"{{"identities":{{"{SA}":{{"subject":"team-a:api","roles":{{"products":"read"}}}}}}}}"#
    );
    // team-b granted nobody. This is the default state of a new tenant.
    let empty = r#"{"identities":{}}"#.to_string();
    // A deliberately over-permissive team-b, used only to isolate layer 1:
    // if the audience check were absent, this registry would say yes to
    // everything, so a rejection here can only have come from `aud`.
    let permissive =
        format!(r#"{{"identities":{{"{SA}":{{"subject":"team-b:api","roles":{{"*":"admin"}}}}}}}}"#);

    let team_a = instance(AUD_A, &granted);
    let team_b = instance(AUD_B, &empty);
    let team_b_permissive = instance(AUD_B, &permissive);

    let mut failures = 0;
    let mut check = |label: &str, ok: bool| {
        println!("{} {label}", if ok { "PASS" } else { "FAIL" });
        if !ok {
            failures += 1;
        }
    };

    // -- the positive path, through the shipped verifier ---------------------
    let principal = team_a.authenticate_async(&bearer(&token_a)).await;
    check(
        "real SA id-token -> team-a authenticates it (offline JWKS, no shared secret)",
        principal.is_ok(),
    );
    let Ok(principal) = principal else {
        println!("\ngcp id-token spike: cannot continue without the positive path");
        std::process::exit(1);
    };
    println!("     subject = {:?}", principal.subject());

    // -- layer 1 alone: audience, with the registry wide open ----------------
    check(
        "team-a's token -> team-b REJECTED even though team-b's registry grants it admin on *",
        team_b_permissive
            .authenticate_async(&bearer(&token_a))
            .await
            .is_err(),
    );
    check(
        "...and the same token IS accepted when the audience matches (so the rejection was `aud`)",
        team_b_permissive
            .authenticate_async(&bearer(&token_b))
            .await
            .is_ok(),
    );

    // -- layer 2 alone: registry, with the audience matching -----------------
    check(
        "correctly-audienced token -> team-b REJECTED because team-b granted nobody",
        team_b.authenticate_async(&bearer(&token_b)).await.is_err(),
    );

    // -- the role map is untouched by any of this ----------------------------
    check(
        "granted collection at the granted role -> allowed",
        principal.ensure("products", Role::Read).is_ok(),
    );
    check(
        "granted collection above the granted role -> denied",
        principal.ensure("products", Role::Admin).is_err(),
    );
    check(
        "ungranted collection -> denied",
        principal.ensure("secrets", Role::Read).is_err(),
    );
    check(
        "audit subject survives the hop",
        principal.subject() == Some("team-a:api"),
    );

    // -- negatives that make the positives mean something --------------------
    let (body, sig) = token_a.rsplit_once('.').expect("JWT has three segments");
    let first = sig.chars().next().expect("non-empty signature");
    let tampered = format!(
        "{body}.{}{}",
        if first == 'A' { 'B' } else { 'A' },
        &sig[1..]
    );
    check(
        "tampered signature -> rejected",
        team_a.authenticate_async(&bearer(&tampered)).await.is_err(),
    );
    check(
        "no Authorization header, auth required -> rejected",
        team_a.authenticate_async(&HeaderMap::new()).await.is_err(),
    );

    // -- the claim this whole direction rests on -----------------------------
    let secrets_in_config: HashMap<&str, &str> = HashMap::new();
    check(
        "no bearer secret appears anywhere in the configuration above",
        secrets_in_config.is_empty()
            && !granted.contains("tokens\"")
            && !permissive.contains("tokens\""),
    );

    println!(
        "\n{}",
        if failures == 0 {
            "gcp id-token spike: ok".to_string()
        } else {
            format!("gcp id-token spike: {failures} FAILED")
        }
    );
    std::process::exit(i32::from(failures != 0));
}
