<!-- HANDWRITE-BEGIN gap="sift-ha-operations-document" tracker="1606" reason="Document Sift single-node and Raft replica deployment, backup, restore, and failure recovery." -->
# Sift High Availability Operations

Sift defaults to a single durable node. Its source-of-truth is the raw-event
journal; materialized logs, metrics, traces, errors, audit events, and change
events are derived from it.

## Single node

Render the development instance with `sift k8s instance render --profile dev`.
It runs one StatefulSet replica with a PVC mounted at `/var/lib/sift`. The
standard `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, and `/docs`
endpoints stay on the serving port.

## Replica mode

The current operator and checked-in CRD intentionally admit only one shard with
one replica. Safe multi-replica membership changes remain separate domain work;
do not raise `replicasPerShard` or `voterCount` until that lifecycle is proven.

## Backup and restore

Use the protected live snapshot boundary with a real GCS destination in
production:

```sh
sift backup \
  --url http://sift.sift.svc.cluster.local:7380 \
  --token "$SIFT_BACKUP_TOKEN" \
  --dest gs://example-sift-backups/sift \
  --retention-secs 604800
```

The operator's scheduled CronJob uses the same `GET /admin/backup` endpoint,
runs under the dedicated `<instance>-backup` ServiceAccount, and can load its
admin bearer token from `spec.backup.adminTokenSecret` key `token`. The token is
optional only when Sift auth is off. The runner never mounts the live PVC.

`--data-dir` is a legacy offline-only backup mode. Stop Sift first; never open
the journal from a second process while the service is writing:

```sh
sift backup \
  --data-dir /var/lib/sift \
  --dest file:///recovery/sift \
  --retention-secs 604800
```

`file://` is suitable only for local development and tests. Restore an exact
object while the replacement service is stopped, before restarting it:

```sh
sift restore --data-dir /var/lib/sift --source file:///recovery/sift-backup.json
```

The restore replaces the local snapshot atomically. Multi-replica bootstrap and
catch-up remain unproven domain work and must not be inferred from this 1x1
procedure.

<!-- HANDWRITE-END -->
