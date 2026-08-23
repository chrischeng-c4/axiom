// CODEGEN-BEGIN
//! `LumenFleet` (`lumen.dev/v1alpha1`) — one cluster-scoped object, owned by
//! the platform team, that declares every data-plane namespace and the
//! settings each one gets.
//!
//! ```text
//! LumenFleet (cluster-scoped, applied once into the control plane)
//!   ├─ defaults: <a complete LumenSpec>            platform knowledge
//!   └─ instances[]: { namespace, name?, spec }     app-team knowledge
//!            │
//!            └─ materializes ──▶ Lumen/team-a, Lumen/team-b, …
//! ```
//!
//! ## Why cluster-scoped
//!
//! The fleet's whole purpose is to own objects in *other* namespaces, and a
//! namespaced owner cannot legally do that: Kubernetes rejects a cross-namespace
//! `ownerReference` with an `OwnerRefInvalidNamespace` event and its garbage
//! collector treats the owner as absent — which would delete the very
//! dependents the fleet just created. A cluster-scoped object owning namespaced
//! dependents is the supported direction, so the fleet is cluster-scoped. That
//! does not weaken "all configuration lives in the control plane": it is still
//! one object only the platform team has RBAC to touch.
//!
//! ## Why a generator, and not a replacement for `Lumen`
//!
//! Each entry still materializes a real `Lumen` CR rather than being reconciled
//! straight into StatefulSets. That keeps `kubectl get lumen -A`, the
//! per-instance `status.conditions[]` (#2601), independent failure domains, and
//! every existing operator behaviour exactly as they are; the fleet only
//! decides which `Lumen` objects should exist and what their specs say.
//!
//! ## What the fleet deliberately does not manage
//!
//! `spec.shardCount`, `spec.shardMap`, and `spec.reshardPolicy.workflow` are
//! written at runtime by the autonomous reshard driver
//! ([`super::reshard_driver`]). A declarative applier that listed them would
//! revert a completed split on its next pass — resetting `shardMap.version`
//! and re-triggering migration over data that has already moved. So the fleet
//! seeds them **once**, on the create that brings an instance into existence,
//! and never names them again: the steady-state apply omits those paths
//! entirely, which under server-side apply means the fleet never owns them and
//! therefore can never remove them.

use std::collections::BTreeSet;
use std::time::Duration;

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::crd::{Lumen, LumenSpec};

/// Label every `Lumen` a fleet materializes carries, naming the fleet that
/// owns it. This — not an `ownerReference` — is the ownership link, on
/// purpose: an owner reference would make deleting the fleet cascade-delete
/// every data plane and its PVCs, so a typo in one cluster-scoped object would
/// destroy every tenant's data. Pruning is explicit and policy-driven instead
/// (see [`PrunePolicy`]).
pub const FLEET_LABEL: &str = "lumen.dev/fleet";

/// Server-side-apply field manager for the steady-state apply.
pub const FLEET_MANAGER: &str = "lumen-fleet";

/// Field manager for the one-time create. Deliberately *not* [`FLEET_MANAGER`]:
/// the initial topology fields it writes must stay owned by a manager that
/// never applies again, so the steady-state apply-set — which omits them —
/// cannot cause the API server to prune them.
pub const FLEET_SEED_MANAGER: &str = "lumen-fleet-seed";

/// Leader-election Lease for the fleet loop, independent of the main
/// controller's so each can fail over on its own.
const FLEET_LEASE_NAME: &str = "lumen-fleet";

/// How often the fleet re-materializes its instances.
const FLEET_POLL_INTERVAL: Duration = Duration::from_secs(30);

/// Spec paths the reshard driver owns at runtime; see the module docs.
const DRIVER_OWNED_PATHS: &[&[&str]] = &[
    &["shardCount"],
    &["shardMap"],
    &["reshardPolicy", "workflow"],
];

/// `lumen.dev/v1alpha1` `LumenFleet`. Cluster-scoped — see the module docs.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "lumen.dev",
    version = "v1alpha1",
    kind = "LumenFleet",
    plural = "lumenfleets",
    shortname = "lfleet",
    status = "LumenFleetStatus",
    printcolumn = r#"{"name":"Desired","type":"integer","jsonPath":".status.desiredInstances"}"#,
    printcolumn = r#"{"name":"Applied","type":"integer","jsonPath":".status.appliedInstances"}"#,
    printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct LumenFleetSpec {
    /// The complete `Lumen` spec every instance starts from — the platform
    /// team's knowledge: which image, which node pool, which StorageClass,
    /// which auth mode. Required and fully schema-validated, because a fleet
    /// that cannot produce one valid instance is not a fleet.
    pub defaults: LumenSpec,

    /// One entry per data-plane namespace.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<FleetInstance>,

    /// What happens to an instance whose entry is removed from
    /// [`Self::instances`].
    #[serde(default)]
    pub prune_policy: PrunePolicy,
}

/// One data plane the fleet declares.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FleetInstance {
    /// The namespace the `Lumen` is materialized into. The namespace must
    /// already exist: creating namespaces from a CR would make the operator's
    /// ClusterRole a namespace-creation privilege, and namespace lifecycle
    /// (quotas, labels, Workload Identity bindings) belongs to whatever
    /// provisions the cluster.
    pub namespace: String,

    /// The `Lumen` object's name. Defaults to the fleet's own name, so
    /// `kubectl get lumen -A` reads as one fleet spread across namespaces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// A JSON Merge Patch (RFC 7386) applied over
    /// [`LumenFleetSpec::defaults`] — the app team's knowledge: this tenant's
    /// CPU/memory request, its disk size, the name of its credential source.
    /// A `null` value removes an inherited field.
    ///
    /// Free-form rather than an enumerated override struct so it covers every
    /// `Lumen` field, now and after the next one is added. Typos cannot hide
    /// in it: the merged document is deserialized into a `LumenSpec` and any
    /// key the spec does not have is reported as a rejected entry rather than
    /// silently dropped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(schema_with = "free_form_object")]
    pub spec: Option<Value>,
}

/// What happens to a materialized `Lumen` whose entry leaves the fleet.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum PrunePolicy {
    /// Leave it running and report it as orphaned. The default, because
    /// removing a line from a list is a plausible edit and deleting a search
    /// index with its PVCs is not a plausible consequence of one.
    #[default]
    Retain,
    /// Delete it, PVCs included via the instance's own garbage collection.
    /// Opt-in only.
    Delete,
}

/// Status subresource for a fleet.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LumenFleetStatus {
    /// The `.metadata.generation` this status reflects.
    #[serde(default)]
    pub observed_generation: i64,
    /// How many instances the spec declares.
    #[serde(default)]
    pub desired_instances: i32,
    /// How many were successfully created or applied this pass.
    #[serde(default)]
    pub applied_instances: i32,
    /// Per-entry outcome, so one bad entry is diagnosable without reading
    /// operator logs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<FleetEntryStatus>,
    /// Human-readable summary of the last pass.
    #[serde(default)]
    pub message: String,
}

/// One entry's outcome.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FleetEntryStatus {
    pub namespace: String,
    pub name: String,
    /// `Created` | `Applied` | `Rejected` | `NamespaceMissing` | `NotAdopted`
    /// | `ApplyFailed` | `Orphaned` | `Pruned`.
    pub state: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub message: String,
}

/// A `Lumen` the fleet wants to exist, or the reason one entry cannot produce
/// one.
#[derive(Clone, Debug, PartialEq)]
pub struct PlannedInstance {
    pub namespace: String,
    pub name: String,
    pub outcome: PlanOutcome,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlanOutcome {
    /// The merged spec, as JSON. Kept as JSON rather than a `LumenSpec` so the
    /// apply body is exactly what was validated, with no second round-trip
    /// between the check and the write.
    Ready(Value),
    /// Why this entry produced nothing.
    Rejected(String),
}

/// The `x-kubernetes-preserve-unknown-fields` object schema for
/// [`FleetInstance::spec`]. A structural CRD schema has to say *something*
/// about every property, and "an object whose keys are validated later, by
/// deserializing the merge result into a `LumenSpec`" is what this expresses.
fn free_form_object(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    let mut schema = schemars::schema::SchemaObject {
        instance_type: Some(schemars::schema::InstanceType::Object.into()),
        ..Default::default()
    };
    schema
        .extensions
        .insert("x-kubernetes-preserve-unknown-fields".to_string(), json!(true));
    schemars::schema::Schema::Object(schema)
}

/// Turn one fleet into the list of `Lumen` specs it declares, without touching
/// a cluster. Every rejection is per-entry: one malformed override must not
/// stop the other tenants from converging.
pub fn plan(fleet: &LumenFleet) -> Vec<PlannedInstance> {
    let fleet_name = fleet.metadata.name.clone().unwrap_or_default();
    let defaults = serde_json::to_value(&fleet.spec.defaults).unwrap_or(Value::Null);
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut planned = Vec::new();

    for entry in &fleet.spec.instances {
        let name = entry.name.clone().unwrap_or_else(|| fleet_name.clone());
        let namespace = entry.namespace.clone();

        // Two entries writing the same object would each revert the other on
        // every pass, and the losing tenant's settings would depend on list
        // order. Reject the duplicate rather than let the fleet oscillate.
        if !seen.insert((namespace.clone(), name.clone())) {
            planned.push(PlannedInstance {
                namespace,
                name,
                outcome: PlanOutcome::Rejected(
                    "a previous entry already targets this namespace/name; \
                     two entries writing one object would revert each other every pass"
                        .to_string(),
                ),
            });
            continue;
        }

        let mut merged = defaults.clone();
        if let Some(patch) = &entry.spec {
            merge_patch(&mut merged, patch);
        }
        planned.push(PlannedInstance {
            namespace,
            name,
            outcome: validate_merged(merged),
        });
    }
    planned
}

/// Deserialize the merged document into a `LumenSpec` and prove nothing in it
/// was silently ignored.
fn validate_merged(merged: Value) -> PlanOutcome {
    // Through `serde_path_to_error` so the rejection names the field. Plain
    // serde would report `invalid type: integer, expected a string` for a
    // wrong `serving.cpu`, leaving whoever reads the fleet's status to guess
    // which of ~40 spec fields is meant.
    let spec: LumenSpec = match serde_path_to_error::deserialize(&merged) {
        Ok(spec) => spec,
        Err(err) => {
            let path = err.path().to_string();
            return PlanOutcome::Rejected(format!(
                "merged spec is not a valid Lumen at `{path}`: {}",
                err.into_inner()
            ));
        }
    };
    let round_trip = match serde_json::to_value(&spec) {
        Ok(value) => value,
        Err(err) => return PlanOutcome::Rejected(format!("merged spec does not re-serialize: {err}")),
    };
    let mut unknown = Vec::new();
    unknown_keys(&merged, &round_trip, "", &mut unknown);
    if !unknown.is_empty() {
        return PlanOutcome::Rejected(format!(
            "the merged spec names fields a Lumen does not have: {}; \
             a misspelled override would otherwise leave this instance silently on the defaults",
            unknown.join(", ")
        ));
    }
    if let Err(err) = spec.validate() {
        return PlanOutcome::Rejected(err);
    }
    PlanOutcome::Ready(merged)
}

/// RFC 7386 JSON Merge Patch: objects merge key-by-key, `null` deletes, every
/// other value replaces wholesale.
fn merge_patch(base: &mut Value, patch: &Value) {
    let Value::Object(patch) = patch else {
        *base = patch.clone();
        return;
    };
    if !base.is_object() {
        *base = Value::Object(Default::default());
    }
    let map = base.as_object_mut().expect("just made it an object");
    for (key, value) in patch {
        if value.is_null() {
            map.remove(key);
        } else {
            merge_patch(map.entry(key.clone()).or_insert(Value::Null), value);
        }
    }
}

/// Keys present in `input` that a `LumenSpec` round-trip dropped — i.e. fields
/// the spec does not have.
///
/// Empty collections and explicit nulls are exempt because `LumenSpec` skips
/// serializing them, so their absence from the round-trip is the serializer's
/// doing rather than evidence of a typo.
fn unknown_keys(input: &Value, round_trip: &Value, prefix: &str, out: &mut Vec<String>) {
    let (Value::Object(input), Value::Object(round_trip)) = (input, round_trip) else {
        return;
    };
    for (key, value) in input {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        match round_trip.get(key) {
            Some(known) => unknown_keys(value, known, &path, out),
            None => {
                let vacuous = value.is_null()
                    || value.as_array().is_some_and(|items| items.is_empty())
                    || value.as_object().is_some_and(|map| map.is_empty());
                if !vacuous {
                    out.push(path);
                }
            }
        }
    }
}

/// The body of the one-time create: the whole merged spec, initial topology
/// included.
pub fn seed_object(fleet: &str, planned: &PlannedInstance, spec: &Value) -> Value {
    json!({
        "apiVersion": "lumen.dev/v1alpha1",
        "kind": "Lumen",
        "metadata": {
            "name": planned.name,
            "namespace": planned.namespace,
            "labels": fleet_labels(fleet),
        },
        "spec": spec,
    })
}

/// The body of every apply after the first: the merged spec **minus** the
/// paths the reshard driver owns. Omitting them from the apply-set is what
/// keeps the fleet from ever reverting a completed split — see the module
/// docs.
pub fn apply_object(fleet: &str, planned: &PlannedInstance, spec: &Value) -> Value {
    let mut spec = spec.clone();
    for path in DRIVER_OWNED_PATHS {
        remove_path(&mut spec, path);
    }
    seed_object(fleet, planned, &spec)
}

fn remove_path(value: &mut Value, path: &[&str]) {
    let Some((head, rest)) = path.split_first() else {
        return;
    };
    let Some(map) = value.as_object_mut() else {
        return;
    };
    if rest.is_empty() {
        map.remove(*head);
    } else if let Some(child) = map.get_mut(*head) {
        remove_path(child, rest);
    }
}

fn fleet_labels(fleet: &str) -> Value {
    json!({
        FLEET_LABEL: fleet,
        "app.kubernetes.io/managed-by": FLEET_MANAGER,
    })
}

// <HANDWRITE gap="missing-generator:logic:async-anchor" tracker="#1855" reason="AW cannot scaffold an async controller loop; the fleet's cluster I/O is bounded by hand around the pure plan/apply functions above.">
/// Run the fleet materialization loop alongside the main controller.
///
/// A poll loop rather than a `kube` `Controller`: the fleet is one
/// cluster-scoped object edited by a human, its children are `Lumen` CRs that
/// have their own controller, and a 30s convergence pass is far below the
/// latency anyone can perceive on a deploy. Leader-gated on its own Lease so a
/// failover of the main controller does not stall it and vice versa.
pub fn spawn_fleet_loop(client: kube::Client) {
    // Same identity/namespace resolution as every other independently
    // leader-gated loop in this operator.
    let identity = std::env::var("POD_NAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| FLEET_LEASE_NAME.to_string());
    let namespace =
        std::env::var("POD_NAMESPACE").unwrap_or_else(|_| "lumen-operator-system".to_string());
    let election = super::lease::Election::new(identity);
    super::lease::spawn(
        client.clone(),
        namespace,
        FLEET_LEASE_NAME.to_string(),
        election.clone(),
    );
    tokio::spawn(async move {
        let fleets: kube::Api<LumenFleet> = kube::Api::all(client.clone());
        loop {
            if election
                .is_leader
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                match fleets.list(&Default::default()).await {
                    Ok(list) => {
                        for fleet in list.items {
                            if let Err(err) = converge(&client, &fleets, &fleet).await {
                                tracing::warn!(
                                    fleet = %fleet.metadata.name.clone().unwrap_or_default(),
                                    error = %err,
                                    "fleet convergence pass failed"
                                );
                            }
                        }
                    }
                    Err(err) => tracing::warn!(error = %err, "fleet: list LumenFleet failed"),
                }
            }
            tokio::time::sleep(FLEET_POLL_INTERVAL).await;
        }
    });
}

/// One convergence pass over a single fleet.
async fn converge(
    client: &kube::Client,
    fleets: &kube::Api<LumenFleet>,
    fleet: &LumenFleet,
) -> anyhow::Result<()> {
    use kube::ResourceExt;

    let fleet_name = fleet.name_any();
    let planned = plan(fleet);
    let mut entries = Vec::new();
    let mut applied = 0;
    let mut declared: BTreeSet<(String, String)> = BTreeSet::new();

    for instance in &planned {
        let spec = match &instance.outcome {
            PlanOutcome::Ready(spec) => spec,
            PlanOutcome::Rejected(reason) => {
                entries.push(entry(instance, "Rejected", reason));
                continue;
            }
        };
        declared.insert((instance.namespace.clone(), instance.name.clone()));

        if !namespace_exists(client, &instance.namespace).await? {
            entries.push(entry(
                instance,
                "NamespaceMissing",
                "the namespace does not exist; the fleet does not create namespaces",
            ));
            continue;
        }

        let lumens: kube::Api<Lumen> = kube::Api::namespaced(client.clone(), &instance.namespace);
        match lumens.get_opt(&instance.name).await? {
            None => {
                let body: Lumen = serde_json::from_value(seed_object(&fleet_name, instance, spec))?;
                let params = kube::api::PostParams {
                    field_manager: Some(FLEET_SEED_MANAGER.to_string()),
                    ..Default::default()
                };
                match lumens.create(&params, &body).await {
                    Ok(_) => {
                        applied += 1;
                        entries.push(entry(instance, "Created", ""));
                    }
                    Err(err) => {
                        entries.push(entry(instance, "ApplyFailed", &err.to_string()));
                    }
                }
            }
            Some(live) => {
                // A `Lumen` this fleet did not create is not this fleet's to
                // overwrite: a hand-authored instance at the same name would
                // otherwise be silently replaced by the fleet's defaults.
                let owner = live.labels().get(FLEET_LABEL).map(String::as_str);
                if owner != Some(fleet_name.as_str()) {
                    entries.push(entry(
                        instance,
                        "NotAdopted",
                        &format!(
                            "a Lumen already exists here and is not labelled {FLEET_LABEL}={fleet_name} \
                             (found {}); left untouched",
                            owner.unwrap_or("no label")
                        ),
                    ));
                    continue;
                }
                let body = apply_object(&fleet_name, instance, spec);
                match lumens
                    .patch(
                        &instance.name,
                        &kube::api::PatchParams::apply(FLEET_MANAGER).force(),
                        &kube::api::Patch::Apply(&body),
                    )
                    .await
                {
                    Ok(_) => {
                        applied += 1;
                        entries.push(entry(instance, "Applied", ""));
                    }
                    Err(err) => entries.push(entry(instance, "ApplyFailed", &err.to_string())),
                }
            }
        }
    }

    entries.extend(prune(client, fleet, &declared).await?);

    let status = json!({
        "status": {
            "observedGeneration": fleet.metadata.generation.unwrap_or(0),
            "desiredInstances": planned.len() as i32,
            "appliedInstances": applied,
            "entries": entries,
            "message": format!("{applied}/{} instances converged", planned.len()),
        }
    });
    fleets
        .patch_status(
            &fleet_name,
            &kube::api::PatchParams::default(),
            &kube::api::Patch::Merge(&status),
        )
        .await?;
    Ok(())
}

/// Report — and, only under [`PrunePolicy::Delete`], remove — instances this
/// fleet created that its spec no longer declares.
async fn prune(
    client: &kube::Client,
    fleet: &LumenFleet,
    declared: &BTreeSet<(String, String)>,
) -> anyhow::Result<Vec<FleetEntryStatus>> {
    use kube::ResourceExt;

    let fleet_name = fleet.name_any();
    let lumens: kube::Api<Lumen> = kube::Api::all(client.clone());
    let params =
        kube::api::ListParams::default().labels(&format!("{FLEET_LABEL}={fleet_name}"));
    let mut out = Vec::new();
    for live in lumens.list(&params).await?.items {
        let namespace = live.namespace().unwrap_or_default();
        let name = live.name_any();
        if declared.contains(&(namespace.clone(), name.clone())) {
            continue;
        }
        let orphan = PlannedInstance {
            namespace: namespace.clone(),
            name: name.clone(),
            outcome: PlanOutcome::Rejected(String::new()),
        };
        match fleet.spec.prune_policy {
            PrunePolicy::Retain => out.push(entry(
                &orphan,
                "Orphaned",
                "no longer declared by this fleet; retained (set prunePolicy: Delete to remove)",
            )),
            PrunePolicy::Delete => {
                let scoped: kube::Api<Lumen> = kube::Api::namespaced(client.clone(), &namespace);
                match scoped.delete(&name, &Default::default()).await {
                    Ok(_) => out.push(entry(&orphan, "Pruned", "")),
                    Err(err) => out.push(entry(&orphan, "ApplyFailed", &err.to_string())),
                }
            }
        }
    }
    Ok(out)
}

async fn namespace_exists(client: &kube::Client, namespace: &str) -> anyhow::Result<bool> {
    let api: kube::Api<k8s_openapi::api::core::v1::Namespace> = kube::Api::all(client.clone());
    Ok(api.get_opt(namespace).await?.is_some())
}

fn entry(instance: &PlannedInstance, state: &str, message: &str) -> FleetEntryStatus {
    FleetEntryStatus {
        namespace: instance.namespace.clone(),
        name: instance.name.clone(),
        state: state.to_string(),
        message: message.to_string(),
    }
}
// </HANDWRITE>

/// The `LumenFleet` CustomResourceDefinition as YAML.
pub fn fleet_crd_yaml() -> String {
    use kube::CustomResourceExt;
    let mut crd = serde_json::to_value(LumenFleet::crd()).expect("CRD serializes to JSON");
    service_k8s::crd::normalize_unsigned_integer_formats(&mut crd);
    serde_yaml::to_string(&crd).expect("CRD serializes")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> LumenSpec {
        serde_json::from_value(json!({
            "image": "ghcr.io/acme/lumen:0.4.27",
            "serving": { "cpu": "2", "memory": "8Gi", "raftStorage": "50Gi" },
            "placement": { "nodeSelector": { "cloud.google.com/gke-nodepool": "lumen-ssd" } },
        }))
        .expect("defaults parse")
    }

    fn fleet(instances: Vec<FleetInstance>) -> LumenFleet {
        let mut fleet = LumenFleet::new(
            "search",
            LumenFleetSpec {
                defaults: defaults(),
                instances,
                prune_policy: PrunePolicy::default(),
            },
        );
        fleet.metadata.generation = Some(1);
        fleet
    }

    fn instance(namespace: &str, spec: Option<Value>) -> FleetInstance {
        FleetInstance {
            namespace: namespace.to_string(),
            name: None,
            spec,
        }
    }

    fn ready(planned: &PlannedInstance) -> &Value {
        match &planned.outcome {
            PlanOutcome::Ready(spec) => spec,
            PlanOutcome::Rejected(reason) => panic!("expected a ready instance, got: {reason}"),
        }
    }

    fn rejection(planned: &PlannedInstance) -> &str {
        match &planned.outcome {
            PlanOutcome::Rejected(reason) => reason,
            PlanOutcome::Ready(_) => panic!("expected a rejection"),
        }
    }

    /// The whole point of `defaults`: a tenant that names nothing still gets
    /// the platform team's image, node pool, and disk.
    #[test]
    fn an_entry_inherits_every_default_it_does_not_name() {
        let planned = plan(&fleet(vec![instance("team-a", None)]));
        let spec = ready(&planned[0]);
        assert_eq!(spec["image"], json!("ghcr.io/acme/lumen:0.4.27"));
        assert_eq!(spec["serving"]["cpu"], json!("2"));
        assert_eq!(
            spec["placement"]["nodeSelector"]["cloud.google.com/gke-nodepool"],
            json!("lumen-ssd")
        );
        assert_eq!(planned[0].namespace, "team-a");
        assert_eq!(planned[0].name, "search", "name defaults to the fleet's");
    }

    /// A merge patch, not a replacement: raising one tenant's CPU must not
    /// silently drop the memory and disk it never mentioned.
    #[test]
    fn an_override_replaces_only_the_field_it_names() {
        let planned = plan(&fleet(vec![instance(
            "team-b",
            Some(json!({ "serving": { "cpu": "8" } })),
        )]));
        let spec = ready(&planned[0]);
        assert_eq!(spec["serving"]["cpu"], json!("8"));
        assert_eq!(spec["serving"]["memory"], json!("8Gi"), "inherited");
        assert_eq!(spec["serving"]["raftStorage"], json!("50Gi"), "inherited");
    }

    /// RFC 7386's `null`: the only way to *unset* something the defaults set.
    #[test]
    fn a_null_override_clears_an_inherited_field() {
        let with_log_level = LumenFleet::new(
            "search",
            LumenFleetSpec {
                defaults: serde_json::from_value(json!({
                    "image": "img", "logLevel": "warn",
                }))
                .unwrap(),
                instances: vec![instance("team-c", Some(json!({ "logLevel": null })))],
                prune_policy: PrunePolicy::default(),
            },
        );
        let planned = plan(&with_log_level);
        assert!(ready(&planned[0]).get("logLevel").is_none());
    }

    /// The failure mode a free-form override has to be defended against: a
    /// misspelled key is accepted by serde, ignored, and the tenant quietly
    /// runs on the default it thought it had changed.
    #[test]
    fn a_misspelled_override_is_rejected_rather_than_ignored() {
        let planned = plan(&fleet(vec![instance(
            "team-d",
            Some(json!({ "serving": { "cpuu": "8" } })),
        )]));
        let reason = rejection(&planned[0]);
        assert!(reason.contains("serving.cpuu"), "{reason}");
    }

    /// Empty collections round-trip away because `LumenSpec` skips
    /// serializing them; the typo check must not read that as a typo.
    #[test]
    fn an_empty_collection_is_not_mistaken_for_an_unknown_field() {
        let planned = plan(&fleet(vec![instance(
            "team-e",
            Some(json!({
                "shardMap": { "assignments": [] },
                "placement": { "tolerations": [] },
            })),
        )]));
        ready(&planned[0]);
    }

    /// A wrong *value* is caught by the same pass, with the field named.
    #[test]
    fn a_value_the_schema_rejects_names_the_field() {
        let planned = plan(&fleet(vec![instance(
            "team-f",
            // `off` is the env spelling; the CRD spells it `disabled`.
            Some(json!({ "auth": "off" })),
        )]));
        assert!(rejection(&planned[0]).contains("auth"));
    }

    /// The retired registry fields (#2872) must not survive as a fleet
    /// override either. The fleet is the one path that takes free-form spec
    /// JSON, so a platform team that kept the old grants in their defaults
    /// would otherwise get them merged in and dropped without a word — the
    /// exact silent no-op the retirement exists to prevent.
    #[test]
    fn a_retired_registry_override_is_rejected_at_the_fleet_too() {
        for retired in [
            json!({ "tokensSecret": "lumen-tokens" }),
            json!({ "identities": { "svc@proj.iam.gserviceaccount.com": { "subject": "team-g" } } }),
            json!({ "identityAudiences": ["https://lumen.example.com"] }),
        ] {
            let key = retired.as_object().unwrap().keys().next().unwrap().clone();
            let planned = plan(&fleet(vec![instance("team-g", Some(retired))]));
            let reason = rejection(&planned[0]);
            assert!(reason.contains(&key), "{reason}");
            assert!(
                reason.contains("a Lumen does not have"),
                "a retired field is now an unknown field, not a validate() rule: {reason}"
            );
        }
    }

    /// One tenant's bad edit must not stop every other tenant from converging.
    #[test]
    fn one_rejected_entry_does_not_stop_the_others() {
        let planned = plan(&fleet(vec![
            instance("team-h", Some(json!({ "serving": { "nope": 1 } }))),
            instance("team-i", None),
        ]));
        assert!(matches!(planned[0].outcome, PlanOutcome::Rejected(_)));
        assert!(matches!(planned[1].outcome, PlanOutcome::Ready(_)));
    }

    /// Two entries writing one object would each revert the other every pass,
    /// so which settings a tenant ends up with would depend on list order.
    #[test]
    fn two_entries_targeting_the_same_object_are_rejected() {
        let mut second = instance("team-j", None);
        second.name = Some("search".to_string());
        let planned = plan(&fleet(vec![instance("team-j", None), second]));
        assert!(matches!(planned[0].outcome, PlanOutcome::Ready(_)));
        assert!(rejection(&planned[1]).contains("already targets"));
    }

    /// ★ The one that protects live data. The reshard driver writes
    /// `shardCount`, `shardMap`, and `reshardPolicy.workflow` at runtime; a
    /// fleet apply that listed them would revert a finished split on its next
    /// pass — resetting the map version and re-migrating data that already
    /// moved.
    #[test]
    fn the_steady_state_apply_never_claims_a_field_the_reshard_driver_owns() {
        let planned = plan(&fleet(vec![instance("team-k", None)]));
        let spec = ready(&planned[0]);
        let applied = apply_object("search", &planned[0], spec);

        assert!(applied["spec"].get("shardCount").is_none(), "{applied}");
        assert!(applied["spec"].get("shardMap").is_none(), "{applied}");
        assert!(
            applied["spec"]["reshardPolicy"].get("workflow").is_none(),
            "{applied}"
        );
        // The rest of `reshardPolicy` is fleet-declared policy and must stay.
        assert!(applied["spec"]["reshardPolicy"]["prepareAtPercent"].is_number());
        assert_eq!(applied["spec"]["image"], json!("ghcr.io/acme/lumen:0.4.27"));
    }

    /// The create is the one moment the initial topology can be declared, so
    /// it carries what the steady-state apply drops.
    #[test]
    fn the_seed_create_carries_the_initial_topology() {
        let planned = plan(&fleet(vec![instance(
            "team-l",
            Some(json!({ "shardCount": 4 })),
        )]));
        let seed = seed_object("search", &planned[0], ready(&planned[0]));
        assert_eq!(seed["spec"]["shardCount"], json!(4));
        assert_eq!(seed["metadata"]["namespace"], json!("team-l"));
        assert_eq!(seed["metadata"]["labels"][FLEET_LABEL], json!("search"));
    }

    /// Ownership is a label, not an `ownerReference` — deleting the fleet must
    /// not cascade-delete every tenant's index and PVCs.
    #[test]
    fn a_materialized_instance_carries_no_owner_reference() {
        let planned = plan(&fleet(vec![instance("team-m", None)]));
        let seed = seed_object("search", &planned[0], ready(&planned[0]));
        assert!(seed["metadata"].get("ownerReferences").is_none(), "{seed}");
        assert_eq!(seed["metadata"]["labels"][FLEET_LABEL], json!("search"));
    }

    /// A namespaced fleet could not legally own objects in other namespaces:
    /// Kubernetes rejects cross-namespace owner references and its garbage
    /// collector deletes the dependents. Cluster scope is the load-bearing
    /// property of this CRD, so it is pinned.
    #[test]
    fn the_fleet_crd_is_cluster_scoped() {
        let yaml = fleet_crd_yaml();
        assert!(yaml.contains("scope: Cluster"), "{yaml}");
        assert!(yaml.contains("lumenfleets"), "{yaml}");
    }

    /// The override is a free-form object, which a structural CRD schema only
    /// accepts with this extension — without it the API server prunes every
    /// key of every override and each tenant silently gets the bare defaults.
    #[test]
    fn the_override_survives_structural_schema_pruning() {
        let yaml = fleet_crd_yaml();
        assert!(
            yaml.contains("x-kubernetes-preserve-unknown-fields"),
            "{yaml}"
        );
        // The defaults are fully schema-validated, so the platform team's own
        // typo is still caught at `kubectl apply`.
        assert!(yaml.contains("shardCount"), "{yaml}");
    }
}
// CODEGEN-END
