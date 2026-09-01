# Deploy Lumen

## Choose a mode

| Mode | Use it for | Lifecycle owner |
|---|---|---|
| Standalone | Local work, tests, and small single-process use. | The caller starts and configures one process or container. |
| Managed | Stateful Kubernetes operation. | The operator reconciles declared instances. |

Fleet is the default Managed entry point. It is a management scope. It does not
enable HA or autoscaling by itself.

## Standalone quickstart

### Binary

```bash
lumen serve
```

The default endpoint is `http://127.0.0.1:7373`. The default index is in memory.
Add `--data-dir <path>` when the index must survive a process restart.

A bare `lumen serve` remains loopback and in-memory unless `--data-dir` or
`LUMEN_DATA_DIR` is set.

The shipped image defaults are `LUMEN_DATA_DIR=/var/lib/lumen/data`,
`LUMEN_PERSISTENCE=segment`, `LUMEN_WAL=embedded`, and
`VOLUME ["/var/lib/lumen/data"]`.

A 0.4.28 volume upgrades only when it contains a valid segment checkpoint in the exact legacy `gen-<seq>` form. On the first 0.4.29 start, Lumen validates the highest such generation and atomically writes `CURRENT`. A corrupt highest legacy generation fails startup and never falls back to an older generation.

A truly new empty root can initialize an empty `CURRENT` baseline. An AOF-only root is also valid when it has a regular `aof.log`; it starts from the empty checkpoint baseline and replays its AOF. `aof.log.compact.tmp` is valid only beside that regular `aof.log`. A compact temporary file on its own fails closed.

`CURRENT` is the checkpoint authority. A valid `CURRENT=empty` remains authoritative even if an unpointed generation beside it looks like data that an operator expected Lumen to restore. Lumen does not choose another generation or rewrite that `CURRENT` automatically. Repair that state from a verified backup or an explicit recovery procedure.

A non-empty root without `CURRENT` that has an unknown layout, a symlink, an invalid entry type, or an unpointed revision generation fails before Lumen writes `CURRENT` or starts its listener. The process logs every direct root entry, its kind, and the refusal reason. It does not open the HTTP listener. A successful segment start logs `segment checkpoint startup decision` with one of `initialized_empty_root`, `recovered_uncommitted_empty`, `restored_current_empty`, `restored_current_generation`, or `adopted_legacy_0428`. AOF recovery logs `AOF startup decision` with `aof_decision=no_tail` or `aof_decision=tail_replayed`.

After 0.4.29 writes or adopts `CURRENT`, in-place downgrade to 0.4.28 is unsupported. Keep a pre-upgrade volume copy or backup for rollback.

### Container

Every Lumen release publishes multi-arch container images to GitHub Packages (GHCR)
at `ghcr.io/chrischeng-c4/lumen`. Semver and `latest` tags are discovery-only
references. Production and verifiable environments should resolve and pin the
immutable root image index digest:

```bash
RAW_DIGEST="$(docker buildx imagetools inspect ghcr.io/chrischeng-c4/lumen:<version> --format '{{json .Manifest}}' | jq -er '.digest')"
[[ "$RAW_DIGEST" =~ ^sha256:[0-9a-f]{64}$ ]] || { echo "invalid digest" >&2; exit 1; }
IMAGE="ghcr.io/chrischeng-c4/lumen@${RAW_DIGEST}"
printf '%s\n' "$IMAGE" > lumen-image.ref
```

Create one protected annotated `lumen@<version>` tag at the exact candidate
commit before promotion. Then verify the public release before deployment:

```bash
apps/lumen/scripts/verify-release-artifacts.sh \
  --repo chrischeng-c4/axiom \
  --tag lumen@<version> \
  --commit <commit> \
  --candidate-run-id <id> \
  --mode public \
  --output /tmp/lumen-public-release.json
```

Each release image carries a keyless Cosign signature and SLSA v1 provenance on the
root index, with per-platform SPDX 2.3 JSON SBOM attestations. The release workflow
verifies artifacts and runs native amd64 and arm64 kind runs before publication. These
local kind runs do not prove GKE, Regional HA, Fleet, or autoscaling.

Shipped container images set `LUMEN_HOST=0.0.0.0` by default so published
ports are reachable from the host without extra environment variables. The bare
binary continues to default to `127.0.0.1` and in-memory storage. The shipped
image defaults to segment storage at its VOLUME path. With no mount at
`/var/lib/lumen/data`, Docker supplies an anonymous volume. `docker run --rm`
removes that anonymous volume with the container. Deliberate replacement
retention uses a named volume or a caller-managed bind mount at the exact data
path:

```bash
docker run --rm -p 127.0.0.1:7373:7373 \
  -e LUMEN_AUTH=off \
  "$IMAGE"
```

Use a named volume mounted directly at `/var/lib/lumen/data`, or a
caller-managed bind mount at that exact path, when local data must survive
container replacement:

```bash
docker volume create lumen-data

docker run -d --name lumen -p 127.0.0.1:7373:7373 \
  --mount type=volume,src=lumen-data,dst=/var/lib/lumen/data \
  -e LUMEN_AUTH=off \
  -e LUMEN_GRACE_SECS=1 \
  "$IMAGE"
```

Stop the container cleanly before replacing it with the same named volume:

```bash
docker stop --time=10 lumen
docker rm lumen

docker run -d --name lumen -p 127.0.0.1:7373:7373 \
  --mount type=volume,src=lumen-data,dst=/var/lib/lumen/data \
  -e LUMEN_AUTH=off \
  -e LUMEN_GRACE_SECS=1 \
  "$IMAGE"
```

The checked-in Compose patch is this storage-only YAML excerpt:

```yaml
services:
  lumen:
    volumes:
      - lumen-data:/var/lib/lumen/data
volumes:
  lumen-data: {}
```

Use the public patch command for a caller-owned Compose file:

```bash
lumen standalone compose patch --file <compose.yaml> [--name <name>]
```

It refuses an existing same-name service unless it has the label
`com.axiom.lumen.managed: 'true'`. It does not update the checked-in
`apps/lumen/compose.yaml` application.

Compose and GKE backup/restore runs must use the administrative restore path
with `--replace`. Restore without `--replace` is refused. Keep backup storage
separate from the runtime PVC or named volume.

Use the public commands for local backups and restores:

```bash
lumen standalone backup --compose apps/lumen/compose.yaml --out /tmp/lumen.snapshot
lumen standalone restore --compose apps/lumen/compose.yaml --file /tmp/lumen.snapshot --replace
lumen standalone backup --gke lumen.yaml --out /tmp/lumen-gke.snapshot
lumen standalone restore --gke lumen.yaml --file /tmp/lumen-gke.snapshot --replace
```

For a GKE runtime, the CLI port-forwards to the Service, makes a 600-second
Kubernetes-default-audience TokenRequest for `<name>-admin`, and keeps that
token only in memory. An app uses only
`LUMEN_URL=http://<name>.<namespace>.svc.cluster.local:7373`.

The container process listens on every container interface by default. The
published host port still listens only on `127.0.0.1`.

Check the process:

```bash
curl -fsS http://127.0.0.1:7373/healthz
curl -fsS http://127.0.0.1:7373/readyz
curl -fsS http://127.0.0.1:7373/openapi.json
```

Use [configuration](configuration.md) for precedence and restart behavior.

## Managed prerequisites

Managed mode uses an existing Kubernetes cluster. Prepare these inputs before
creating an instance:

- A cluster that can run the operator and StatefulSets.
- Target namespaces for every Fleet entry. The operator does not create them.
- The legacy GCE capacity catalog at `lumen-system/lumen-capacity-catalog`
  when using an empty selector, tolerations-only placement, or a non-default
  `placement.initialMachineType`.
- A catalog entry compatible with each legacy instance's
  `placement.initialMachineType`. A non-empty `placement.nodeSelector` with
  the default machine type uses the compatibility path and keeps the selector
  and tolerations from the manifest.
- A serving TLS Secret when `servingTlsSecret` is set.
- A peer TLS Secret when `replicasPerShard` is greater than one.
- Any StorageClass, ServiceAccount, monitoring CRDs, and backup destination
  named by the spec.

Managed request auth also needs client-side work. A client workload needs an
audience-bound projected ServiceAccount token and code that adds it to each
HTTP request. Attaching a ServiceAccount to a pod does not perform either step.

TLS Secrets contain `tls.crt`, `tls.key`, and `ca.crt`. The current operator
does not create a certificate or Secret from a Fleet declaration. The
deployment platform owns that material today. The target operator will request
and rotate separate serving and peer leaf certificates from a platform-owned
issuer. It will publish public serving trust for allowed client namespaces.

The checked-in operator bundle creates its own `lumen-system` namespace. A
custom `--namespace` value changes the rendered control-plane namespace. It
does not create target namespaces.

See the [Terraform guide](../terraform/README.md) for the current GCP capacity
and certificate substrate.

### Production target

GKE Standard Regional is the first Managed production target. Current
acceptance uses zonal GKE Standard only. It does not prove regional HA. The
future profile uses Kubernetes-native resources, StorageClass, selectors,
tolerations, and topology intent. GKE-only ComputeClass and node lifecycle stay
in the platform profile, not the core Lumen API. See the [GKE guide](gke.md).

## Install the Managed control plane

Use one released `lumen` binary to render matching CRDs and operator resources.

```bash
mkdir -p /tmp/lumen-install

lumen k8s crd render --out /tmp/lumen-install
lumen k8s operator render \
  --namespace lumen-system \
  --image "$IMAGE" \
  --out /tmp/lumen-install

kubectl apply -f /tmp/lumen-install/crd.yaml
kubectl apply -f /tmp/lumen-install/operator.yaml
```

Confirm the two operator replicas. Confirm the capacity catalog too when the
instance uses the legacy placement path:

```bash
kubectl -n lumen-system rollout status deploy/lumen-operator
kubectl -n lumen-system get pods -l app.kubernetes.io/name=lumen-operator
kubectl -n lumen-system get configmap lumen-capacity-catalog
```

## Create a Managed Fleet

Render a fill-in template:

```bash
lumen k8s fleet render \
  --profile template \
  --out /tmp/lumen-install
```

Edit `/tmp/lumen-install/lumenfleet.yaml`. One minimal shape is:

```yaml
apiVersion: lumen.dev/v1alpha1
kind: LumenFleet
metadata:
  name: search
spec:
  prunePolicy: Retain
  defaults:
    image: ghcr.io/chrischeng-c4/lumen@sha256:<64-lowercase-root-digest>
    auth: required
    servingTlsSecret: search-serving-tls
    serving:
      cpu: "2"
      memory: 8Gi
      raftStorage: 50Gi
      raftStorageClass: ssd
    placement:
      initialMachineType: e2-standard-2
  instances:
    - namespace: search-team-a
      spec:
        serving:
          cpu: "4"
    - namespace: search-team-b
```

`instances[].spec` is an RFC 7386 merge patch over `defaults`. It is not a
typed runtime patch. A `null` value removes an inherited optional field.

The planned `defaults.access` and `instances[].access` fields are not in the
current CRD. Do not add them to a current Fleet manifest. See the
[authentication guide](authentication.md#planned-whole-runtime-access) for the
target contract.

Apply the declaration:

```bash
kubectl apply -f /tmp/lumen-install/lumenfleet.yaml
kubectl get lumenfleet search -o yaml
kubectl get lumen -A -l lumen.dev/fleet=search
```

Current Fleet entry states such as `Created` and `Applied` confirm child
materialization only. Check each child `Lumen` condition and StatefulSet before
declaring the runtime ready:

```bash
kubectl get lumen -A -l lumen.dev/fleet=search
kubectl -n search-team-a get lumen search -o yaml
kubectl -n search-team-a wait \
  --for=condition=Ready lumen/search \
  --timeout=10m
kubectl -n search-team-a rollout status statefulset/search
```

## Use a direct Lumen resource

Direct `Lumen` is the advanced Managed entry point. Use it when one instance
must be declared outside Fleet management.

```bash
lumen k8s instance render \
  --profile staging \
  --namespace search-team-a \
  --name search \
  --out /tmp/lumen-install

kubectl apply -f /tmp/lumen-install/lumen.yaml
```

The same current namespace, legacy capacity catalog, certificate, storage, and
identity prerequisites apply.

## Direct kustomize compatibility

The checked-in overlays are a Standalone runtime template. They run one
Standalone Deployment inside Kubernetes.

```bash
kubectl apply -k apps/lumen/k8s/overlays/dev
```

This path is single-process and in-memory. The staging and prod overlay names
select configuration examples. They do not turn the Deployment into Managed
mode, HA, durable storage, or autoscaling. Use the operator for those
StatefulSet controls.

## Client access

### Current access bundle

When `auth: required`, a client uses a short-lived Kubernetes ServiceAccount
token. The current Fleet does not render client access RBAC. Render the current
manual bundle with:

```bash
lumen k8s access render \
  --namespace search-team-a \
  --client-sa app-client \
  --issuer alice@example.com \
  --grant docs=read \
  --out /tmp/lumen-install

kubectl apply -f /tmp/lumen-install/access.yaml
```

This command renders five objects. They include the named client
ServiceAccount, the grant that lets the issuer request its token, and the
current per-collection Role and RoleBinding that Lumen checks.

The issuer can then use the existing developer flow:

```bash
lumen connect \
  --namespace search-team-a \
  --cr search \
  --client-sa app-client \
  --ca-file /path/to/search-serving-ca.crt \
  -- lumen query collections list
```

`lumen connect` uses kubeconfig identity for TokenRequest. It keeps the token
inside the process and adds it through a loopback proxy. Kubernetes RBAC
decides whether that issuer may mint the token.

An application workload must currently project a token for audience
`lumen.axiom.dev`, read the rotated file, and set the Authorization header
itself. Generated clients do not do this automatically. Never place a bearer
token in a Fleet, environment variable, process argument, status, Event, or
log.

### Planned Fleet access

The planned Managed contract declares exact allowed client ServiceAccounts in
`Lumen.spec.access`, `LumenFleet.spec.defaults.access`, or the replacing
`instances[].access` list. The operator then converges whole-runtime Roles and
RoleBindings. This contract and `AccessPolicyReady` are not implemented.

Fleet will not create a client ServiceAccount, namespace, Deployment, token
Secret, or TokenRequest issuer grant. The application or platform must still
project the standard token into the client workload. See
[authentication](authentication.md) for the full current and planned flow. The
[client integration guide](client-integration.md) owns the future explicit
connection profiles and versioned workload template.

## Upgrade order

Apply Managed changes in this order:

1. CRDs.
2. Operator image and RBAC.
3. Fleet or direct `Lumen` declarations.
4. Current manual client-access resources when their contract changed.

The API currently serves `lumen.dev/v1alpha1`. Additive fields can remain in
that version. No conversion webhook exists. Do not downgrade the CRD after
stored resources use newer fields.

The new Fleet rollout policy is not implemented yet. Current Fleet convergence
can apply every entry in one pass. Plan a controlled operator and Fleet change
until [safe rollout](../ROADMAP.md#fleet-safe-rollout) lands.

Fleet rollout will order changes between runtimes. A separate
[quorum-safe runtime rollout](../ROADMAP.md#quorum-safe-runtime-rollout) must
control members within one runtime. A PDB protects voluntary eviction. It does
not stop a StatefulSet rolling update.

The planned typed access API and 0.5.0 Managed auth requirements are also not
implemented. Follow the [migration contract](authentication.md#migration-contract)
before changing current external RBAC.

The planned 0.5 search contract adds a separate Managed activation boundary.
Installing a new binary will not be enough. Every serving member must report
the required binary capability. The operator must then finalize
`compatibilityVersion` before `search_facets_v1` becomes active. A
mixed-version runtime will reject activation. It will not route new requests
around an older member.

Current `/version` and the current operator do not implement this capability
model. See [Managed activation](migration-0.5-search.md#managed-activation)
before planning the 0.5 upgrade.

## Smoke checks

Run these checks for each changed instance:

```bash
kubectl -n lumen-system get lease lumen-operator
kubectl get lumenfleet -o wide
kubectl get lumen -A
kubectl -n <namespace> get statefulset,pod,service,pvc
kubectl -n <namespace> get lumen <name> -o yaml
kubectl -n <namespace> port-forward service/<name> 7373:7373
```

In another shell:

```bash
curl -fsS http://127.0.0.1:7373/healthz
curl -fsS http://127.0.0.1:7373/readyz
curl -fsS http://127.0.0.1:7373/version
```

If serving TLS is enabled, use the matching CA and Service DNS name. A normal
localhost request cannot verify a certificate issued only for cluster Service
names.

Use the [operator runbook](runbooks/operator-control-plane.md) for Fleet,
capacity, child-status, leadership, and reconcile diagnostics.

The future GKE Standard Regional gate also covers node drain, Pod loss, zone
loss, rollout interruption, recovery, certificate rotation, client trust,
backup, and restore. Do not use the current zonal result as that evidence.
