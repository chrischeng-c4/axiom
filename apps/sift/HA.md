# Sift high availability operations

Sift has one product boundary and several runtime roles. Every role uses the
same Rust binary and the same `/var/lib/sift` data-root rules.

## Small installation

Run `sift serve --role all` for local use, Docker, or a small installation.
This role is persistent by default. Mount a durable volume at `/var/lib/sift`.

The `all` role is one failure domain. A process restart recovers its WAL and
projections, but one lost volume loses the local copy. Use GCS archives and
backups when the recovery point must survive a lost node or disk.

## GKE role topology

One `Sift` resource renders these workloads:

- gateway Deployment
- query Deployment
- three-replica store StatefulSet
- three-replica control StatefulSet
- agent DaemonSet
- optional backup CronJob

The client Service selects the gateway. The gateway forwards HTTP and
OTLP/gRPC ingest to the store. It forwards queries to the query role. The query
role reads from the store source of truth.

Store and control each use one fixed three-voter Raft group. Two durable voters
must commit a write before Sift reports success. Live membership changes and a
different voter count are refused.

Each pod that owns local state has its own PVC at `/var/lib/sift`. Do not use an
`emptyDir` for Sift data. The init container sets the owner and mode `0700`.
The workload runs as a non-root user. The pod security context uses
`fsGroupChangePolicy: OnRootMismatch`.

## Peer TLS

Raft traffic uses mutual TLS on port `7381`. It does not share the public API
port.

Set `spec.peerTlsSecret` to a Secret with these keys:

- `tls.crt`
- `tls.key`
- `ca.crt`

The certificate must permit both TLS client and server use. Its server names
must cover every pod name below both headless Services:

```text
<instance>-store-{0,1,2}.<instance>-store-headless.<namespace>.svc
<instance>-control-{0,1,2}.<instance>-control-headless.<namespace>.svc
```

Sift refuses replicated startup when mutual TLS is off or any file is absent.

## Kubernetes delegated authentication

Set `spec.auth: kubernetes` to use TokenReview and SubjectAccessReview. The
runtime ServiceAccount is named after the Sift instance. The operator creates
an instance-scoped ClusterRoleBinding to the built-in
`system:auth-delegator` role. The operator can bind only that role. It installs
a finalizer before creating the binding. It deletes the binding before auth is
disabled or the Sift resource is deleted. It also reads the ready
`default/kubernetes` Endpoints object and limits review traffic to those IPs.
Reconcile fails before workloads start if no ready API endpoint exists. Sift
checks the delegated grant at process startup and fails closed when it is
absent.

The agent and backup job use a projected ServiceAccount token with audience
`sift.axiom.dev`. Sift rereads the token file for every request. This supports
normal Kubernetes token rotation.

## Backup and archive

Use the protected live snapshot boundary for online backup:

```sh
sift backup \
  --url http://<instance>.<namespace>.svc.cluster.local:7380 \
  --token-file /var/run/secrets/sift/client/token \
  --token-audience sift.axiom.dev \
  --project <instance> \
  --dest gs://example-sift-backups/sift \
  --retention-secs 604800
```

The operator can render the same command in a CronJob. The job does not mount a
serving PVC.

`sift backup --data-dir` is offline-only. Stop the process that owns the data
root before using it:

```sh
sift backup \
  --data-dir /var/lib/sift \
  --dest file:///recovery/sift \
  --retention-secs 604800
```

Restore an exact backup while the replacement Sift process is stopped:

```sh
sift restore \
  --data-dir /var/lib/sift \
  --source file:///recovery/sift-backup.json
```

Signal archives use immutable Snappy-compressed Parquet objects. Sift writes
the archive manifest last. It compacts the corresponding WAL only after that
manifest is committed. If GCS is unavailable, Sift keeps the local WAL and
returns backpressure before the configured local capacity limit.

## Verification boundary

The local `raft_failover` test starts three durable voters with mutual TLS. It
commits data, stops the leader, elects another leader, and commits more data.
The surviving quorum retains both writes.

This is not a live GKE recovery result. An MVP candidate still needs the
dedicated 30-minute, 10,000-item-per-second GKE run. That run includes PVC
pod recreation, voter failover, a GCS outage, and fresh-PVC restore. Production
high availability still requires the later 24-hour and 100,000-item-per-second
gates.
