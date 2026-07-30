// HANDWRITE-BEGIN gap="missing-generator:logic:e9e1ff60" tracker="pending-tracker" reason="impl ManagedService for Tape: MANAGER tape-operator (SSA field manager + leader-election Lease name); render() -> render::render; prunes() -> render::prunes; readiness_targets = [StatefulSet {name}]; status_patch + conditions() both project from one Observation (Ready/Progressing/StorageHealthy/BackupConfigured) so the flat phase and the conditions can never disagree; observed_conditions() round-trips status.conditions for lastTransitionTime; pub async fn run() = service_k8s::run::<Tape>()."
//! tape's operator wiring onto the shared `libs/service-k8s` controller.
//!
//! The reconcile loop + leader-election lease live in `libs/service-k8s`
//! (`service_k8s::run` drives the watch + leader-gated server-side apply over
//! kube-rs). tape supplies only its [`ManagedService`] impl — what to render,
//! which children a previous spec rendered but this one no longer wants, which
//! workload to poll for readiness, and the `Tape` status subresource
//! (flat fields + per-concern conditions) to write.

use kube::ResourceExt;
use serde_json::json;
use service_k8s::{ConditionFact, ConditionStatus, ManagedService, ReadinessTarget, ReadyFacts};

use super::crd::Tape;
use super::render;

impl ManagedService for Tape {
    /// Server-side-apply field manager + leader-election Lease name.
    const MANAGER: &'static str = "tape-operator";

    fn render(&self) -> Vec<serde_json::Value> {
        render::render(self)
    }

    /// #3054: prune the objects tape conditionally renders (the observability
    /// ServiceMonitor/PrometheusRule pair, the backup CronJob) when the CR
    /// stops asking for them.
    fn prunes(&self) -> Vec<service_k8s::service::PruneTarget> {
        render::prunes(self)
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        // tape is always a StatefulSet (durable journal + raft state on a
        // PVC); poll it for `.status.readyReplicas`.
        vec![ReadinessTarget {
            kind: "StatefulSet",
            name: self.name_any(),
        }]
    }

    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value {
        let obs = self.observe(ready);
        json!({ "status": {
            "phase": obs.phase,
            "observedGeneration": self.metadata.generation.unwrap_or(0),
            "readyReplicas": obs.ready_replicas,
            "desiredReplicas": obs.desired,
            "message": obs.replicas_message(),
        }})
    }

    /// #3054: the Kubernetes-convention convergence surface, derived from the
    /// same [`Observation`] [`Self::status_patch`] projects — so the flat
    /// `phase` and the conditions can never disagree about whether this CR
    /// has converged.
    ///
    /// Clock-free by construction: the caller stamps `lastTransitionTime`,
    /// which is what keeps this side of the projection deterministic (see the
    /// module doc's no-I/O `status_patch` contract).
    fn conditions(&self, ready: &ReadyFacts, _context: &serde_json::Value) -> Vec<ConditionFact> {
        self.observe(ready).conditions(self)
    }

    /// #3054: the conditions already persisted on this object, so the shared
    /// projection can carry each `lastTransitionTime` forward. `Patch::Merge`
    /// replaces arrays wholesale, so nothing survives server-side unless it is
    /// read back off the watched object and re-sent.
    fn observed_conditions(&self) -> Vec<service_k8s::Condition> {
        self.status
            .as_ref()
            .map(|status| status.conditions.clone())
            .unwrap_or_default()
    }
}

/// One reconcile's worth of observed facts about a `Tape` (#3054).
///
/// Both status surfaces — the flat legacy fields and `status.conditions[]` —
/// project from this single computation rather than each re-deriving the
/// phase, so they cannot drift apart.
struct Observation {
    ready_replicas: i32,
    desired: i32,
    phase: &'static str,
}

impl Observation {
    /// The same `"{ready}/{desired} tape pods ready"` string both
    /// `status_patch.message` and the `Ready` condition's message use.
    fn replicas_message(&self) -> String {
        format!("{}/{} tape pods ready", self.ready_replicas, self.desired)
    }

    /// Read off `phase` rather than re-testing `ready_replicas >= desired`.
    /// Spelling the predicate a second time would reintroduce exactly the
    /// drift this struct exists to prevent: the flat `status.phase` and the
    /// `Ready` condition would be two expressions that agree only as long as
    /// whoever edits one remembers the other.
    fn replicas_ready(&self) -> bool {
        self.phase == "Ready"
    }

    /// The clock-free condition facts for this observation, in printed order:
    /// `Ready`, `Progressing`, `StorageHealthy`, `BackupConfigured`.
    fn conditions(&self, tape: &Tape) -> Vec<ConditionFact> {
        let replicas = self.replicas_message();
        let ready = if self.replicas_ready() {
            ConditionFact::new(
                "Ready",
                ConditionStatus::True,
                "AllReplicasReady",
                replicas.clone(),
            )
        } else {
            ConditionFact::new(
                "Ready",
                ConditionStatus::False,
                "ReplicasNotReady",
                replicas.clone(),
            )
        };

        let progressing = if self.replicas_ready() {
            ConditionFact::new(
                "Progressing",
                ConditionStatus::False,
                "Converged",
                "spec is fully reconciled".to_string(),
            )
        } else {
            ConditionFact::new(
                "Progressing",
                ConditionStatus::True,
                "ReplicasConverging",
                replicas,
            )
        };

        // #3054 AC4 wanted `False` here for a degraded instance. The operator
        // cannot see that fact, for three independent reasons: the sticky
        // ENOSPC flag (#2573) lives in the server process
        // (`src/server.rs::is_storage_degraded`) and leaves it only as the
        // `tape_storage_degraded` gauge; `/readyz` deliberately stays green
        // while degraded — serving reads is the whole point of "degraded
        // read-only" — so `ReadyFacts` carries no trace either; and tape
        // implements no `reconcile_plan`, so `conditions`'s `context`
        // argument is always `Value::Null`.
        //
        // A condition reporting `False` for a reason unrelated to real
        // storage health is worse than one that admits it has not looked, so
        // this reports `Unknown`/`NotObserved` and names where the fact *is*
        // visible today rather than inventing a probe, an HTTP scrape, or any
        // other proxy signal. #3054's own Out of Scope excluded that work
        // ("any genuinely new probe is separate work"); it is tracked as
        // #3071, which adds the `reconcile_plan` observation path this
        // condition would project from.
        let storage_healthy = ConditionFact::new(
            "StorageHealthy",
            ConditionStatus::Unknown,
            "NotObserved",
            "the operator has no observation path to the per-node ENOSPC \
             degraded-storage state (#2573); see the tape_storage_degraded \
             gauge and the TapeStorageDegraded alert for the live signal"
                .to_string(),
        );

        let backup_configured = match &tape.spec.backup {
            Some(backup) => ConditionFact::new(
                "BackupConfigured",
                ConditionStatus::True,
                "ScheduleConfigured",
                format!("scheduled '{}' to {}", backup.schedule, backup.destination),
            ),
            None => ConditionFact::new(
                "BackupConfigured",
                ConditionStatus::False,
                "NotConfigured",
                "spec.backup is unset".to_string(),
            ),
        };

        vec![ready, progressing, storage_healthy, backup_configured]
    }
}

impl Tape {
    /// Compute this reconcile's [`Observation`] — the single source both
    /// status surfaces project from (#3054). Synchronous and I/O-free, per
    /// the module doc's contract.
    fn observe(&self, ready: &ReadyFacts) -> Observation {
        let name = self.name_any();
        let ready_replicas = ready.get(&name) as i32;
        // tape is a single raft group: shardCount is pinned to 1 by the
        // render, so replicasPerShard is the desired replica count.
        let desired = self.spec.cluster.replicas_per_shard as i32;
        let phase = if desired > 0 && ready_replicas >= desired {
            "Ready"
        } else if ready_replicas > 0 {
            "Reconciling"
        } else {
            "Pending"
        };
        Observation {
            ready_replicas,
            desired,
            phase,
        }
    }
}

/// `tape k8s operator run` — run the reconcile controller on the shared
/// `libs/service-k8s` host (leader-gated; safe at `replicas > 1`).
pub async fn run() -> anyhow::Result<()> {
    service_k8s::run::<Tape>().await
}
// HANDWRITE-END
