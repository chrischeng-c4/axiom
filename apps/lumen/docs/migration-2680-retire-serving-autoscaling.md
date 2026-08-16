# Migration: Retiring `spec.serving.autoscaling` (#2680)

## Summary

The `spec.serving.autoscaling` block (`minReplicas`, `maxReplicas`, `targetCpuUtilization`) has been removed from the `Lumen` and `LumenFleet` CRD specifications (`lumen.dev/v1alpha1`).

## Rationale

Lumen data plane instances run stateful workloads with durable PVC storage and Raft replication. Standard Kubernetes `HorizontalPodAutoscaler` (HPA) cannot safely manage stateful data-plane capacity or execute Raft membership transitions. The `spec.serving.autoscaling` fields were inert and advertised HPA behavior that did not run.

Capacity and topology decisions are managed directly via `spec.shardCount` and `spec.replicasPerShard`.

## Required Actions for Operators

1. **Update Manifests**: Remove the `autoscaling` block under `spec.serving` from all `Lumen` custom resources and `LumenFleet` instance overrides.

   Before:
   ```yaml
   apiVersion: lumen.dev/v1alpha1
   kind: Lumen
   metadata:
     name: search
     namespace: default
   spec:
     image: lumen:latest
     serving:
       cpu: "2"
       memory: "4Gi"
       autoscaling:
         minReplicas: 2
         maxReplicas: 6
         targetCpuUtilization: 70
   ```

   After:
   ```yaml
   apiVersion: lumen.dev/v1alpha1
   kind: Lumen
   metadata:
     name: search
     namespace: default
   spec:
     image: lumen:latest
     serving:
       cpu: "2"
       memory: "4Gi"
   ```

2. **Schema Pruning and Fleet Validation**: The two deployment paths handle retired fields differently:
   - **Direct `Lumen` CRs**: The Kubernetes API server structural schema silently prunes unrecognized fields without returning an error. Because pruning is silent and nothing prompts the deployer, manifest cleanup is the operator's own responsibility; otherwise, a stale `autoscaling` block sits in repository manifests looking effective forever.
   - **`LumenFleet` instance overrides**: The fleet planner (`plan`) validates merged overrides via deserialization and round-trip diff (`unknown_keys`). Any field `LumenSpec` does not have is caught and marked `Rejected` with a status reason explicitly naming `serving.autoscaling`.

3. **Stale HPA Cleanup**: Upgrading the operator will not strand existing HPA objects. The operator retains its background garbage collector to automatically prune any legacy HorizontalPodAutoscaler objects left behind by older versions.
