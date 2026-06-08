"""Hot-loop bench for `google.cloud.pubsub.PublisherClient` /
`google.cloud.pubsub.SubscriberClient` /
`google.cloud.pubsub.SchemaServiceClient` /
`google.cloud.pubsub.__version__` module-attribute reads (#1511).

End-user scenario: google-cloud-pubsub-using services re-resolve
`google.cloud.pubsub.PublisherClient` (publisher),
`google.cloud.pubsub.SubscriberClient` (subscriber),
`google.cloud.pubsub.SchemaServiceClient` (schema service), and
`google.cloud.pubsub.__version__` (version string sentinel) on
every call site. Per-call attribute resolution goes through the
`google.cloud.pubsub` module's attribute table on each call site.
That per-call module-attribute quadruple-read is the workload
measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `PublisherClient`,
`SubscriberClient`, `SchemaServiceClient`, and `__version__` per
iteration (ITERS scaled so 4 attrs x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import google.cloud.pubsub


_P_BASELINE = google.cloud.pubsub.PublisherClient
_S_BASELINE = google.cloud.pubsub.SubscriberClient
_SS_BASELINE = google.cloud.pubsub.SchemaServiceClient
_VERSION_BASELINE = google.cloud.pubsub.__version__

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = google.cloud.pubsub.PublisherClient
    b = google.cloud.pubsub.SubscriberClient
    c = google.cloud.pubsub.SchemaServiceClient
    d = google.cloud.pubsub.__version__
    if (a is _P_BASELINE
            and b is _S_BASELINE
            and c is _SS_BASELINE
            and d is _VERSION_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"google.cloud.pubsub module-attribute read acc drift: acc={acc} expected={ITERS}"
print("google_cloud_pubsub_type_read_hot:", acc)
