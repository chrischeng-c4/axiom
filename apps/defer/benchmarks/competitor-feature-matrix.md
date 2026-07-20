# Defer competitor feature matrix

Defer is scoped against managed HTTP push queues. Google Cloud Tasks is the
semantic competitor; language worker frameworks such as Celery and Sidekiq
are explicitly outside the comparison because they do not own the same HTTP
push service contract.

| Capability | Cloud Tasks | Defer | Product decision |
|---|---|---|---|
| Future schedule / ETA | yes | yes | core |
| HTTP target dispatch | native | native | core |
| Success deletes/completes task | 2xx | 2xx | core |
| At-least-once retry | yes | yes | core; stable idempotency key is sent on every attempt |
| Queue rate / burst / max in-flight | yes | yes | committed globally across replicas |
| Pause / resume / disable | yes | yes | core operations |
| Per-task target headers and method | yes | yes | core |
| Target authentication | OAuth/OIDC | HMAC signing | intentionally cloud-neutral |
| Durable HA scheduler state | managed | Raft + per-replica durable state | core |
| DLQ terminal state | queue policy | explicit replicated terminal state | core |
| Task cancellation / inspection | yes | yes | core |
| Force-run bypass | yes | no | excluded: bypassing committed rate/permit policy weakens the service contract |
| Arbitrary language worker execution | no | no | excluded: Relay owns pull workers; Defer owns HTTP push |
| Periodic/cron schedules | separate Scheduler product | no | excluded: scheduler composition belongs outside Defer |
| Workflow/batches | no | no | excluded: Loom owns orchestration |

Reference contracts:

- Google Cloud Tasks HTTP targets and success/retry behavior:
  https://docs.cloud.google.com/tasks/docs/reference/rest/v2/projects.locations.queues.tasks
- Google Cloud Tasks queue rate and concurrency controls:
  https://docs.cloud.google.com/tasks/docs/configuring-queues
The only competitor selected for a formal efficiency claim is Google
Cloud Tasks. Until a real Cloud Tasks queue and publicly reachable target are
available under the same declared region/hardware/network conditions, that
claim remains unproven. The local vat emulator may verify protocol behavior,
but must not be reported as Google Cloud Tasks performance.

Defer's local implementation-efficiency ceiling is its sibling Relay, not a
substitute competitor. Run
`cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture`.
The gate uses identical 128-byte payloads, batch shape, single-voter Raft,
fsync-always durability, and enqueue -> committed lease -> committed ack
lifecycle. Defer may trail Relay by at most 20% because it additionally owns
ETA, queue rate/burst/in-flight permits, attempts, retry/DLQ, and terminal task
state. Passing this gate proves bounded scheduler overhead only; it does not
make a Cloud Tasks performance claim.
