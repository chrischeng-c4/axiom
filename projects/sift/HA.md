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

Set `replicasPerShard` and `voterCount` above one in the Sift custom resource.
The operator injects `POD_NAME`, `SHARD_COUNT`, `REPLICAS_PER_SHARD`, and
`VOTER_COUNT`; Sift then uses `raft-host` topology and the same h2c serving
port for peer Raft RPCs. A write is acknowledged only after the Raft state
machine applies its ordered event to the shared CRC-framed journal.

## Backup and restore

Use an off-node destination in production:

```sh
sift backup --data-dir /var/lib/sift --dest s3://example/sift --retention-secs 604800
```

`file://` is suitable only for local development and tests. Restore an exact
object before restarting a replacement node:

```sh
sift restore --data-dir /var/lib/sift --source file:///recovery/sift-backup.json
```

The restore replaces the local snapshot atomically; replica nodes then recover
their Raft state and catch up from the committed log.

<!-- marker: sift-ha-operations-document path: projects/sift/HA.md reason: Document Sift single-node and Raft replica deployment, backup, restore, and failure recovery. -->
<!-- HANDWRITE-END -->
