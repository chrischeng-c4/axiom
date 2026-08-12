// HANDWRITE-BEGIN gap="missing-generator:logic:bb5e0fdb" tracker="#1849" reason="Own RenderCtx, ServicePodTemplate, labels, owner references, resources, ServiceAccount, ordinary ClusterIP Service, PDB, HPA, and CronJob composition independent of workload kind."
//! Workload-neutral Pod templates and ordinary Kubernetes child helpers.
//!
//! The helpers are re-exported from the existing render root during the first
//! service-k8s landing so StatefulSet consumers keep their source-compatible
//! imports. New workload profiles compose through this semantic module.

use serde_json::{json, Value};

use crate::lifecycle::{
    TerminationBudget, DRAIN_ENDPOINT_PATH, ENV_SERVICE_RUNTIME_DEADLINE_SECONDS,
    ENV_SERVICE_SIGKILL_RESERVE_SECONDS,
};

pub use super::{
    client_service, client_service_with_ports, cron_job, guaranteed_resources,
    horizontal_pod_autoscaler, owner_ref, pdb, requested_resources, service_account, CronJob,
    HorizontalPodAutoscaler, RenderCtx,
};

/// Workload-neutral Pod contract used by the Deployment profile and available
/// to other workload renderers. It deliberately contains no stable identity,
/// PVC, shard, ordinal, peer, or session-affinity fields.
pub struct ServicePodTemplate<'a> {
    pub cx: &'a RenderCtx<'a>,
    pub component: &'a str,
    pub image: &'a str,
    pub image_pull_policy: &'a str,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub ports: Vec<Value>,
    pub env: Vec<Value>,
    pub env_from: Vec<Value>,
    pub resources: Value,
    pub readiness_probe: Option<Value>,
    pub liveness_probe: Option<Value>,
    pub startup_probe: Option<Value>,
    pub lifecycle: Option<Value>,
    pub container_security_context: Option<Value>,
    pub pod_security_context: Option<Value>,
    pub service_account_name: Option<&'a str>,
    pub termination_grace_period_seconds: Option<u64>,
    pub volumes: Vec<Value>,
    pub volume_mounts: Vec<Value>,
    pub pod_annotations: Option<Value>,
    pub topology_spread_constraints: Vec<Value>,
}

impl ServicePodTemplate<'_> {
    /// Apply probes, preStop lifecycle hook, environment variables, and termination grace period derived from a validated [`TerminationBudget`].
    pub fn with_termination_budget(mut self, budget: &TerminationBudget, probe_port: u16) -> Self {
        self.liveness_probe = Some(budget.render_liveness_probe(probe_port));
        self.readiness_probe = Some(budget.render_readiness_probe(probe_port));
        self.startup_probe = Some(budget.render_startup_probe(probe_port));
        self.termination_grace_period_seconds = Some(budget.total_grace_period_seconds());

        if budget.prestop_cost_seconds().is_some() {
            self.lifecycle = Some(json!({
                "preStop": {
                    "httpGet": {
                        "path": DRAIN_ENDPOINT_PATH,
                        "port": probe_port,
                    }
                }
            }));
        }

        let deadline_val = budget.runtime_deadline_seconds().to_string();
        let reserve_val = budget.sigkill_reserve_seconds().to_string();

        let mut deadline_found = false;
        let mut reserve_found = false;

        let mut new_env = Vec::with_capacity(self.env.len() + 2);
        for item in self.env {
            let name = item.get("name").and_then(|n| n.as_str());
            if name == Some(ENV_SERVICE_RUNTIME_DEADLINE_SECONDS) {
                if !deadline_found {
                    deadline_found = true;
                    let mut updated = item.clone();
                    updated["value"] = json!(deadline_val);
                    new_env.push(updated);
                }
            } else if name == Some(ENV_SERVICE_SIGKILL_RESERVE_SECONDS) {
                if !reserve_found {
                    reserve_found = true;
                    let mut updated = item.clone();
                    updated["value"] = json!(reserve_val);
                    new_env.push(updated);
                }
            } else {
                new_env.push(item);
            }
        }

        if !deadline_found {
            new_env.push(json!({
                "name": ENV_SERVICE_RUNTIME_DEADLINE_SECONDS,
                "value": deadline_val,
            }));
        }
        if !reserve_found {
            new_env.push(json!({
                "name": ENV_SERVICE_SIGKILL_RESERVE_SECONDS,
                "value": reserve_val,
            }));
        }

        self.env = new_env;
        self
    }

    /// Render the `spec.template` value shared by workload controllers.
    pub fn render(self) -> Value {
        let mut container = json!({
            "name": self.component,
            "image": self.image,
            "imagePullPolicy": self.image_pull_policy,
            "command": self.command,
            "ports": self.ports,
            "env": self.env,
            "resources": self.resources,
        });
        if !self.args.is_empty() {
            container["args"] = json!(self.args);
        }
        if !self.env_from.is_empty() {
            container["envFrom"] = json!(self.env_from);
        }
        if let Some(probe) = self.readiness_probe {
            container["readinessProbe"] = probe;
        }
        if let Some(probe) = self.liveness_probe {
            container["livenessProbe"] = probe;
        }
        if let Some(probe) = self.startup_probe {
            container["startupProbe"] = probe;
        }
        if let Some(lifecycle) = self.lifecycle {
            container["lifecycle"] = lifecycle;
        }
        if let Some(context) = self.container_security_context {
            container["securityContext"] = context;
        }
        if !self.volume_mounts.is_empty() {
            container["volumeMounts"] = json!(self.volume_mounts);
        }

        let mut pod_spec = json!({ "containers": [container] });
        if let Some(name) = self.service_account_name {
            pod_spec["serviceAccountName"] = json!(name);
        }
        if let Some(seconds) = self.termination_grace_period_seconds {
            pod_spec["terminationGracePeriodSeconds"] = json!(seconds);
        }
        if let Some(context) = self.pod_security_context {
            pod_spec["securityContext"] = context;
        }
        if !self.volumes.is_empty() {
            pod_spec["volumes"] = json!(self.volumes);
        }
        if !self.topology_spread_constraints.is_empty() {
            pod_spec["topologySpreadConstraints"] = json!(self.topology_spread_constraints);
        }

        let mut metadata = json!({ "labels": self.cx.labels(self.component) });
        if let Some(annotations) = self.pod_annotations {
            metadata["annotations"] = annotations;
        }
        json!({ "metadata": metadata, "spec": pod_spec })
    }
}

/// One instance's default-deny network posture, expressed as the two peer
/// classes a sharded service actually has (#2603).
///
/// A NetworkPolicy is deny-by-default *for the pods it selects*: once any
/// policy selects a pod, only the union of matching rules is permitted. That
/// makes the two port lists the whole contract — `client_ports` is the public
/// API surface, `peer_ports` is consensus/replication traffic that must never
/// be reachable from outside the instance.
pub struct NetworkPolicy<'a> {
    pub cx: &'a RenderCtx<'a>,
    pub name: &'a str,
    pub component: &'a str,
    /// Ports any workload in the cluster may reach — the service's client API.
    /// Empty means the instance accepts no ingress at all.
    pub client_ports: Vec<i32>,
    /// Ports only this instance's own pods may reach, in both directions —
    /// Raft, replication, gossip. Empty for a stateless single-pod service.
    pub peer_ports: Vec<i32>,
    /// Egress rules beyond the DNS + TLS baseline, as raw
    /// `networking.k8s.io/v1` egress entries. A service that talks to a
    /// non-443 external dependency (a broker, a database) supplies it here;
    /// most services leave this empty.
    pub extra_egress: Vec<Value>,
}

fn tcp_ports(ports: &[i32]) -> Vec<Value> {
    ports
        .iter()
        .map(|port| json!({ "protocol": "TCP", "port": port }))
        .collect()
}

/// Render the instance's NetworkPolicy.
///
/// The peer rules select on [`RenderCtx::selector`], which includes
/// `app.kubernetes.io/instance` — so two Lumen CRs sharing a namespace cannot
/// reach each other's consensus ports, only their own siblings'.
pub fn network_policy(p: NetworkPolicy<'_>) -> Value {
    let selector = p.cx.selector(p.component);
    let peers = json!({ "podSelector": { "matchLabels": selector } });

    let mut ingress = Vec::new();
    if !p.client_ports.is_empty() {
        // `namespaceSelector: {}` is every namespace, not every source: it
        // still excludes anything outside the pod network (a LoadBalancer's
        // external client reaches the pod through a node, which this rule does
        // not admit). Cluster-internal reach is the intended API posture.
        ingress.push(json!({
            "from": [{ "namespaceSelector": {} }],
            "ports": tcp_ports(&p.client_ports),
        }));
    }
    if !p.peer_ports.is_empty() {
        ingress.push(json!({ "from": [peers], "ports": tcp_ports(&p.peer_ports) }));
    }

    let mut egress = Vec::new();
    if !p.peer_ports.is_empty() {
        egress.push(json!({ "to": [peers], "ports": tcp_ports(&p.peer_ports) }));
    }
    // DNS (both transports — a truncated UDP answer retries over TCP) plus
    // outbound TLS, which is what object-storage backups, OIDC discovery, and
    // image-independent HTTPS calls need. Plaintext :80 is deliberately not
    // granted; a service that needs it declares `extra_egress`.
    egress.push(json!({
        "ports": [
            { "protocol": "UDP", "port": 53 },
            { "protocol": "TCP", "port": 53 },
            { "protocol": "TCP", "port": 443 },
        ],
    }));
    egress.extend(p.extra_egress);

    json!({
        "apiVersion": "networking.k8s.io/v1",
        "kind": "NetworkPolicy",
        "metadata": p.cx.meta(p.name, p.component),
        "spec": {
            "podSelector": { "matchLabels": p.cx.selector(p.component) },
            "policyTypes": ["Ingress", "Egress"],
            "ingress": ingress,
            "egress": egress,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cx() -> RenderCtx<'static> {
        RenderCtx {
            app: "pgpool",
            manager: "pgpool-operator",
            api_version: "pgpool.axiom.dev/v1alpha1",
            kind: "Pgpool",
            name: "pool",
            ns: "database",
            owner: None,
        }
    }

    #[test]
    fn pod_template_preserves_runtime_and_drain_fields() {
        let cx = cx();
        let template = ServicePodTemplate {
            cx: &cx,
            component: "pool",
            image: "pgpool:1",
            image_pull_policy: "IfNotPresent",
            command: vec!["pgpool".into()],
            args: vec!["serve".into()],
            ports: vec![json!({ "name": "postgres", "containerPort": 6432 })],
            env: vec![json!({ "name": "DB_HOST", "value": "remote-db" })],
            env_from: vec![json!({ "secretRef": { "name": "database" } })],
            resources: guaranteed_resources("500m", "512Mi"),
            readiness_probe: Some(json!({ "tcpSocket": { "port": "postgres" } })),
            liveness_probe: Some(json!({ "httpGet": { "path": "/healthz", "port": 9080 } })),
            startup_probe: Some(json!({ "httpGet": { "path": "/healthz", "port": 9080 } })),
            lifecycle: Some(
                json!({ "preStop": { "httpGet": { "path": "/drain", "port": 9080 } } }),
            ),
            container_security_context: Some(json!({ "runAsNonRoot": true })),
            pod_security_context: Some(json!({ "seccompProfile": { "type": "RuntimeDefault" } })),
            service_account_name: Some("pool"),
            termination_grace_period_seconds: Some(60),
            volumes: vec![json!({ "name": "tmp", "emptyDir": {} })],
            volume_mounts: vec![json!({ "name": "tmp", "mountPath": "/tmp" })],
            pod_annotations: Some(json!({ "prometheus.io/scrape": "true" })),
            topology_spread_constraints: vec![json!({
                "maxSkew": 1,
                "topologyKey": "kubernetes.io/hostname",
                "whenUnsatisfiable": "ScheduleAnyway",
                "labelSelector": { "matchLabels": cx.selector("pool") },
            })],
        }
        .render();

        let pod = &template["spec"];
        let container = &pod["containers"][0];
        for key in [
            "command",
            "args",
            "ports",
            "env",
            "envFrom",
            "resources",
            "readinessProbe",
            "livenessProbe",
            "startupProbe",
            "lifecycle",
            "securityContext",
            "volumeMounts",
        ] {
            assert!(!container[key].is_null(), "missing container field {key}");
        }
        for key in [
            "serviceAccountName",
            "terminationGracePeriodSeconds",
            "securityContext",
            "volumes",
            "topologySpreadConstraints",
        ] {
            assert!(!pod[key].is_null(), "missing pod field {key}");
        }
        assert_eq!(
            template["metadata"]["annotations"]["prometheus.io/scrape"],
            "true"
        );
    }

    #[test]
    fn ordinary_children_are_cluster_ip_and_non_sticky() {
        let cx = cx();
        let service = client_service(&cx, "pool", "pool", 6432);
        assert_eq!(service["spec"]["type"], "ClusterIP");
        assert!(service["spec"]["sessionAffinity"].is_null());
        assert_eq!(service_account(&cx, "pool")["kind"], "ServiceAccount");
        assert_eq!(pdb(&cx, "pool", "pool", 1)["spec"]["maxUnavailable"], 1);
    }

    fn policy(peer_ports: Vec<i32>, extra_egress: Vec<Value>) -> Value {
        let cx = cx();
        network_policy(NetworkPolicy {
            cx: &cx,
            name: "pool",
            component: "pool",
            client_ports: vec![6432],
            peer_ports,
            extra_egress,
        })
    }

    #[test]
    fn peer_ports_are_never_reachable_from_outside_the_instance() {
        let policy = policy(vec![9999], vec![]);
        let ingress = policy["spec"]["ingress"].as_array().expect("ingress rules");

        // The whole point of the policy: the consensus port must not appear in
        // any rule whose source is the cluster at large. If it ever does, every
        // pod in every namespace can speak the replication protocol.
        let open_to_cluster = ingress
            .iter()
            .find(|rule| !rule["from"][0]["namespaceSelector"].is_null())
            .expect("client rule");
        assert_eq!(
            open_to_cluster["ports"],
            json!([{"protocol": "TCP", "port": 6432}])
        );

        let peer_rule = ingress
            .iter()
            .find(|rule| !rule["from"][0]["podSelector"].is_null())
            .expect("peer rule");
        assert_eq!(
            peer_rule["ports"],
            json!([{"protocol": "TCP", "port": 9999}])
        );
        // Instance-scoped, not app-scoped: a second Pgpool in this namespace
        // must not be admitted to this one's consensus port.
        assert_eq!(
            peer_rule["from"][0]["podSelector"]["matchLabels"]["app.kubernetes.io/instance"],
            "pool"
        );
    }

    #[test]
    fn egress_baseline_resolves_dns_and_allows_tls_but_not_plaintext() {
        let policy = policy(vec![9999], vec![]);
        let egress = policy["spec"]["egress"].as_array().expect("egress rules");
        let baseline = egress
            .iter()
            .find(|rule| rule["to"].is_null())
            .expect("unrestricted-destination rule");
        assert_eq!(
            baseline["ports"],
            json!([
                { "protocol": "UDP", "port": 53 },
                { "protocol": "TCP", "port": 53 },
                { "protocol": "TCP", "port": 443 },
            ]),
            "a truncated UDP answer retries over TCP/53; plaintext :80 is not granted"
        );
    }

    #[test]
    fn a_service_with_no_peers_emits_no_peer_rules_and_still_denies_by_default() {
        let policy = policy(vec![], vec![]);
        assert_eq!(policy["spec"]["ingress"].as_array().unwrap().len(), 1);
        assert_eq!(policy["spec"]["egress"].as_array().unwrap().len(), 1);
        assert_eq!(
            policy["spec"]["policyTypes"],
            json!(["Ingress", "Egress"]),
            "both directions must stay declared, or the unlisted one is unrestricted"
        );
    }

    #[test]
    fn extra_egress_is_appended_not_substituted() {
        let broker = json!({ "ports": [{ "protocol": "TCP", "port": 4222 }] });
        let policy = policy(vec![9999], vec![broker.clone()]);
        let egress = policy["spec"]["egress"].as_array().expect("egress rules");
        assert_eq!(egress.last(), Some(&broker));
        assert!(
            egress.iter().any(|rule| rule["ports"][0]["port"] == 53),
            "a custom destination must not displace DNS"
        );
    }
}
// HANDWRITE-END
