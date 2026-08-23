// HANDWRITE-BEGIN gap="missing-generator:logic:lumen-certificate-profiles" tracker="#3110" reason="The DNS and SPIFFE identities a Lumen instance may claim are derived from its own topology; every other part of the certificate lifecycle is shared and lives in libs/service-k8s."
//! Which identities a Lumen instance is allowed to claim.
//!
//! This is the whole of Lumen's share of the certificate lifecycle (#3110 AC6).
//! Issuance, renewal timing, trust-bundle overlap, Secret projection, status,
//! and the CA Service requester are all in [`service_k8s::certificate`], because
//! none of them are Lumen-specific and a second service copying them would be
//! how the two drift apart.
//!
//! What *is* Lumen-specific is the answer to "what names does this instance
//! answer to". Serving traffic arrives at one ClusterIP Service. Raft traffic
//! arrives at a pod's own headless DNS name, one per member, and the dialing
//! side verifies that name — so the peer leaf has to enumerate every member the
//! topology can produce. Both facts come from the `Lumen` spec and from nowhere
//! else.
//!
//! ### On enumerating members rather than wildcarding them
//!
//! `*.lumen-headless.lumen.svc.cluster.local` would satisfy the issuing pool's
//! suffix policy (#3109) and would be shorter. It would also certify pods that
//! do not exist and names this instance never serves, which is the opposite of
//! what a private trust domain is for. The list is bounded by the declared
//! topology, and a topology change reissuing the peer leaf is correct: the set
//! of members that may speak Raft genuinely changed.

use std::time::Duration;

use service_k8s::certificate::profile::{
    CertificateIdentity, CertificateProfile, InstanceScope, ProfileError, Purpose,
};

use super::crd::Lumen;

/// Leaf lifetime. Twelve hours, against the issuing pool's 24h ceiling (#3109).
///
/// Short enough that a leaked leaf is a bounded problem, long enough that the
/// controller can be down for a working day without any instance losing its
/// identity.
pub const LEAF_LIFETIME: Duration = Duration::from_secs(12 * 3_600);

/// How early renewal begins: a quarter of the lifetime, which leaves three hours
/// of retries before anything stops working.
pub const RENEW_BEFORE: Duration = Duration::from_secs(3 * 3_600);

/// Spread across the fleet so a hundred instances provisioned in the same
/// minute do not all renew in the same minute a day later. Derived from the
/// leaf's own fingerprint by the shared lifecycle, not from a clock or an RNG,
/// so it survives a controller restart unchanged.
pub const RENEW_JITTER: Duration = Duration::from_secs(30 * 60);

/// Kubernetes' own cluster DNS suffix.
const CLUSTER_DOMAIN: &str = "svc.cluster.local";

/// The scope every certificate for `lumen` is issued under.
///
/// `trust_domain` is the environment's Workload Identity pool
/// (`<project>.svc.id.goog`), supplied by the operator's configuration rather
/// than by the custom resource: it is a property of the cluster, and a spec
/// field for it would let one tenant's CR name another tenant's trust domain.
pub fn scope(lumen: &Lumen, trust_domain: &str) -> InstanceScope {
    InstanceScope::new(namespace(lumen), instance(lumen), trust_domain)
}

/// The certificate the client Service presents.
///
/// `serverAuth` only, and only the two spellings of the ClusterIP Service's own
/// name. Notably absent: any node, LoadBalancer, or external name. Lumen's
/// public edge is not this certificate's problem (#3113 owns that question), and
/// a name added here "just in case" is a name this instance can impersonate.
pub fn serving_profile(
    lumen: &Lumen,
    trust_domain: &str,
) -> Result<CertificateProfile, ProfileError> {
    let name = instance(lumen);
    let ns = namespace(lumen);
    let fqdn = format!("{name}.{ns}.{CLUSTER_DOMAIN}");
    CertificateProfile::new(
        &scope(lumen, trust_domain),
        Purpose::Serving,
        fqdn.clone(),
        CertificateIdentity {
            dns_names: vec![fqdn, format!("{name}.{ns}.svc")],
            spiffe_uri: None,
        },
        LEAF_LIFETIME,
        RENEW_BEFORE,
        RENEW_JITTER,
    )
}

/// The certificate every Raft member presents *and* verifies on `:7374`.
///
/// One identity for the instance, not one per pod, matching the
/// `spec.peerTlsSecret` contract #2890 already deployed: the same three keys are
/// projected into every member. The SPIFFE URI is the instance's workload
/// ServiceAccount, which is what makes "this connection is from my own fleet"
/// checkable rather than "this connection is from something this CA signed".
pub fn peer_profile(lumen: &Lumen, trust_domain: &str) -> Result<CertificateProfile, ProfileError> {
    let name = instance(lumen);
    let ns = namespace(lumen);
    let headless = format!("{name}-headless");
    let mut dns_names = vec![
        format!("{headless}.{ns}.{CLUSTER_DOMAIN}"),
        format!("{headless}.{ns}.svc"),
    ];
    for ordinal in 0..member_count(lumen) {
        dns_names.push(format!("{name}-{ordinal}.{headless}.{ns}.{CLUSTER_DOMAIN}"));
    }
    CertificateProfile::new(
        &scope(lumen, trust_domain),
        Purpose::Peer,
        format!("{headless}.{ns}.{CLUSTER_DOMAIN}"),
        CertificateIdentity {
            dns_names,
            spiffe_uri: Some(format!(
                "spiffe://{trust_domain}/ns/{ns}/sa/{}",
                service_account(lumen)
            )),
        },
        LEAF_LIFETIME,
        RENEW_BEFORE,
        RENEW_JITTER,
    )
}

/// Every StatefulSet ordinal the declared topology can produce.
///
/// `shardCount * replicasPerShard`, because each shard is its own StatefulSet
/// slice of the same fleet. Saturating rather than wrapping: a spec that
/// overflows this should produce a certificate request the CA refuses on size,
/// not a certificate for member zero alone.
fn member_count(lumen: &Lumen) -> u32 {
    lumen
        .spec
        .shard_count
        .max(1)
        .saturating_mul(lumen.spec.replicas_per_shard.max(1))
}

/// The workload ServiceAccount, whether the operator renders it or the deployer
/// brought their own (#2497).
fn service_account(lumen: &Lumen) -> String {
    lumen
        .spec
        .service_account_name
        .clone()
        .unwrap_or_else(|| instance(lumen))
}

fn instance(lumen: &Lumen) -> String {
    lumen
        .metadata
        .name
        .clone()
        .unwrap_or_else(|| "lumen".to_string())
}

fn namespace(lumen: &Lumen) -> String {
    lumen
        .metadata
        .namespace
        .clone()
        .unwrap_or_else(|| "default".to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::operator::crd::{LumenSpec, ServingSpec, ShardMapSpec};

    const TRUST_DOMAIN: &str = "lumen-prod.svc.id.goog";

    fn lumen(shards: u32, replicas: u32) -> Lumen {
        let spec = LumenSpec {
            image: "lumen:latest".into(),
            image_pull_policy: None,
            placement: Default::default(),
            shard_count: shards,
            shard_map: ShardMapSpec::default(),
            replicas_per_shard: replicas,
            voter_count: replicas,
            log_format: Default::default(),
            log_level: None,
            auth: Default::default(),
            serving: ServingSpec::default(),
            reshard_policy: Default::default(),
            observability: false,
            network_policy: false,
            admission: None,
            service_account_name: None,
            service_account_annotations: BTreeMap::new(),
            peer_tls_secret: Some("lumen-peer-tls".into()),
            serving_tls_secret: Some("lumen-serving-tls".into()),
            body_limit_bytes: None,
        };
        let mut lumen = Lumen::new("lumen", spec);
        lumen.metadata.namespace = Some("lumen".to_string());
        lumen
    }

    #[test]
    fn the_serving_leaf_carries_only_the_client_service_names() {
        let profile = serving_profile(&lumen(1, 1), TRUST_DOMAIN).expect("serving profile");
        assert_eq!(
            profile.identity().dns_names,
            vec!["lumen.lumen.svc.cluster.local", "lumen.lumen.svc"]
        );
        assert!(
            profile.identity().spiffe_uri.is_none(),
            "a serving leaf authenticates a Service, not a workload; a SPIFFE URI here would \
             make it usable as a client credential too"
        );
    }

    #[test]
    fn the_peer_leaf_names_every_member_the_topology_can_produce() {
        let profile = peer_profile(&lumen(2, 3), TRUST_DOMAIN).expect("peer profile");
        let names = &profile.identity().dns_names;
        for ordinal in 0..6 {
            let expected = format!("lumen-{ordinal}.lumen-headless.lumen.svc.cluster.local");
            assert!(names.contains(&expected), "missing {expected}: {names:?}");
        }
        assert!(
            !names.iter().any(|name| name.contains("lumen-6.")),
            "certified a member the topology cannot produce: {names:?}"
        );
    }

    #[test]
    fn the_peer_leaf_carries_the_workload_identity() {
        let profile = peer_profile(&lumen(1, 3), TRUST_DOMAIN).expect("peer profile");
        assert_eq!(
            profile.identity().spiffe_uri.as_deref(),
            Some("spiffe://lumen-prod.svc.id.goog/ns/lumen/sa/lumen")
        );
    }

    #[test]
    fn an_externally_managed_service_account_is_the_identity_that_gets_certified() {
        let mut lumen = lumen(1, 3);
        lumen.spec.service_account_name = Some("platform-lumen".into());
        let profile = peer_profile(&lumen, TRUST_DOMAIN).expect("peer profile");
        assert_eq!(
            profile.identity().spiffe_uri.as_deref(),
            Some("spiffe://lumen-prod.svc.id.goog/ns/lumen/sa/platform-lumen"),
            "certifying the SA the operator would have rendered, while the pods run as another \
             one, produces a leaf whose identity nothing in the cluster matches"
        );
    }

    #[test]
    fn certificates_land_in_the_secrets_the_deployment_already_reads() {
        let scope = scope(&lumen(1, 3), TRUST_DOMAIN);
        assert_eq!(
            scope.secret_name(Purpose::Peer),
            "lumen-peer-tls",
            "this is the name `lumen k8s instance render` writes into spec.peerTlsSecret; a \
             different one would need a pod-spec change on every renewal (#3110 R9)"
        );
        assert_eq!(scope.secret_name(Purpose::Serving), "lumen-serving-tls");
    }

    #[test]
    fn the_renewal_window_leaves_room_to_fail() {
        assert!(RENEW_BEFORE < LEAF_LIFETIME);
        assert!(
            RENEW_JITTER < RENEW_BEFORE,
            "jitter wider than the window could push a renewal past expiry"
        );
    }

    #[test]
    fn an_instance_in_another_namespace_gets_that_namespaces_names() {
        let mut lumen = lumen(1, 3);
        lumen.metadata.namespace = Some("staging".into());
        let profile = peer_profile(&lumen, TRUST_DOMAIN).expect("peer profile");
        assert!(profile
            .identity()
            .dns_names
            .iter()
            .all(|name| name.contains(".staging.svc")));
        assert_eq!(
            profile.identity().spiffe_uri.as_deref(),
            Some("spiffe://lumen-prod.svc.id.goog/ns/staging/sa/lumen")
        );
    }

}
// HANDWRITE-END
