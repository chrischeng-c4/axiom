# Configure Lumen

## Scope

This document explains where runtime settings come from and when a change
takes effect. It describes the current source. Planned Managed configuration is
in the [roadmap](../ROADMAP.md#runtime-configuration-parity).

## Standalone precedence

For settings exposed by `lumen serve`, precedence is:

1. A CLI argument.
2. Its `LUMEN_*` environment variable.
3. The built-in default.

For example, `--host 0.0.0.0` overrides `LUMEN_HOST`. Without either value, the
bare binary listens on `127.0.0.1`. Without `LUMEN_AUTH`, Standalone auth is
off.

The public Standalone GKE `lumen.yaml` fields are exactly `name`, `namespace`,
`nodePool`, `cpu`, `memory`, `storageSize`, `storageClass`, and
`allowedServiceAccounts`. It has no image field; the renderer fixes the
published Lumen version.

Use `lumen serve --help` for the maintained flag and environment mapping. The
main groups are:

| Group | Examples | Current behavior |
|---|---|---|
| Listener | `--host`, `LUMEN_HOST`, `--port`, `LUMEN_PORT` | Read at process start. |
| Logging | `--log-level`, `LUMEN_LOG_LEVEL`, `--log-format`, `LUMEN_LOG_FORMAT` | Read at process start. |
| Storage | `--data-dir`, `LUMEN_DATA_DIR`, `LUMEN_PERSISTENCE`, `LUMEN_SNAPSHOT_SECS` | Read at process start. No data directory means in-memory by default. |
| Truncate reclamation | `LUMEN_RECLAIM_WORKERS` | Selects 1 to 4 workers for the one process-wide reclaim queue. The default is 1. Larger values are capped at 4. The value is read when the first truncate starts the reclaimer; restart the process to change it. |
| Replication | `LUMEN_WAL`, `LUMEN_RAFT_DATA_DIR`, `LUMEN_HEADLESS_SERVICE`, `LUMEN_PEERS` | Read at process start. Managed topology adds operator-owned identity values. |
| Security | `LUMEN_AUTH` and serving or peer TLS paths | Auth mode is read at start. TLS files can reload at their mounted paths. |
| Admission and limits | `LUMEN_ADMISSION_*`, `LUMEN_BODY_LIMIT_BYTES` | Read at process start. |
| Observability | `LUMEN_OTLP_ENDPOINT` | Read at process start. |

One reachable local container can use:

```bash
docker run --rm -p 127.0.0.1:7373:7373 \
  -e LUMEN_AUTH=off \
  ghcr.io/chrischeng-c4/lumen:<version>
```

## Managed precedence

The current Fleet path has two user-controlled layers, from lower to higher
priority:

1. `LumenFleet.spec.defaults` supplies a complete base `LumenSpec`.
2. `LumenFleet.spec.instances[].spec` applies an RFC 7386 JSON Merge Patch.

An instance patch replaces only the fields it names. A `null` value removes an
inherited optional field. The Fleet controller validates the merged object as a
`LumenSpec` and rejects unknown or invalid fields for that entry.

`instances[].spec` is not a typed patch at Kubernetes admission time. It also
cannot set a runtime option that has no `LumenSpec` field. General `extraEnv`,
`extraArgs`, and typed `instances[].runtime` fields do not exist yet.

The planned contract keeps Fleet defaults as the lower-priority base. The
advanced `instances[].spec` patch and typed `instances[].runtime` settings are
both instance-level inputs. Neither silently wins over the other. The API will
reject an entry when both inputs set the same runtime value.

The operator owns the final container identity, topology, storage paths,
security paths, command, and required environment values. A direct namespaced
`Lumen` object bypasses Fleet merging, but it uses the same operator rendering
and validation.

Current placement uses a compatibility split. A non-empty
`placement.nodeSelector` with the default `placement.initialMachineType` uses
the Kubernetes-native compatibility path and does not read the capacity
catalog. Empty selectors, tolerations-only placement, and a non-default
machine type use the legacy catalog. These are compatibility inputs. The planned
[Kubernetes-native placement](../ROADMAP.md#kubernetes-native-placement) uses
resource requests, StorageClass, selectors, tolerations, and topology intent.
GKE ComputeClass selection belongs to the GKE profile, not to a portable GCE
machine-type field.

## Authentication configuration

The current Managed path uses the `LumenSpec.auth` value after the normal Fleet
merge. Managed defaults it to `required`, but `disabled` is still accepted.
Client permission is not part of the current Fleet spec. It comes from
externally applied per-collection or instance-admin RBAC.

The planned typed access contract uses a different precedence rule:

1. `LumenFleet.spec.defaults.access` supplies the common complete allow-list.
2. `LumenFleet.spec.instances[].access` replaces that complete list for one
   instance.

The lists do not form a union. An explicit empty list means deny all. Direct
Managed uses `Lumen.spec.access` without Fleet merging.

`instances[].spec.access` will be invalid. It cannot become another route to
the same setting. The planned fields and rejection do not exist in the current
CRD. See [authentication](authentication.md) for the complete contract and
migration boundary.

## Change activation

### Hot reload

Mounted serving TLS and peer TLS files reload at their existing paths. Secret
content rotation does not require an intentional workload rollout.

The current `lumen connect` flow refreshes its TokenRequest credential in
memory. A custom Rust caller can read a `ProjectedTokenFile` for each call.
Generated clients do not yet reread the standard projected token path. The
planned explicit `ManagedKsa` connection profile makes kubelet token rotation
visible on the next request without a pod restart. Standalone mode does not
inspect the token path. See [client integration](client-integration.md).

### Restart required

Most flags and environment variables are read when `lumen serve` starts. A
change must reach a new process to take effect.

Managed rendering currently has no general effective-config hash. Some values
are stored in a ConfigMap and referenced by name from the Pod template. A
ConfigMap data-only change does not itself create a new StatefulSet generation.
The reshard workflow has a focused rolling-restart mechanism for its own shard
map, but that mechanism is not a general runtime-configuration rollout.

### Immutable or special transition

| Setting | Current rule |
|---|---|
| `placement.initialMachineType` | Create-time input for the legacy catalog path. The default value is retained with a non-empty `nodeSelector` on the compatibility path. |
| StatefulSet volume claim template | Kubernetes does not update existing PVC size from a template edit. The resize command can grow an expandable PVC. Shrink is unsupported. |
| `shardCount`, `shardMap`, `reshardPolicy.workflow` after Fleet creation | The Lumen reshard driver owns these paths. Fleet omits them from steady-state apply. |
| Existing collection field type | Drop and recreate the field. Online schema extension only adds fields. |

## Effective configuration visibility

Lumen has no command or endpoint that prints one redacted, fully resolved
effective runtime configuration.

Use these current sources instead:

- `lumen serve --help` for Standalone flags, environment variables, and
  defaults.
- `lumen k8s crd render` for the Managed API schema.
- `kubectl get lumen <name> -n <namespace> -o yaml` for the stored child spec.
- The rendered StatefulSet and ConfigMap for the operator projection.

Do not copy Secret values into a Fleet spec or status. Current TLS and identity
material enters the workload through Kubernetes Secret or projected-token
files.

Do not copy bearer token values into a CLI argument, environment variable,
Fleet field, status, Event, or log. Access declarations contain only
ServiceAccount namespace and name. They never contain a credential.
