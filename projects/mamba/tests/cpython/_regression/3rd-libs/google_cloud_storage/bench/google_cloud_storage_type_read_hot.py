"""Hot-loop bench for `google.cloud.storage.Client` /
`google.cloud.storage.Bucket` /
`google.cloud.storage.Blob` /
`google.cloud.storage.__version__` module-attribute reads (#1510).

End-user scenario: google-cloud-storage-using services re-resolve
`google.cloud.storage.Client` (storage client),
`google.cloud.storage.Bucket` (bucket model),
`google.cloud.storage.Blob` (object model), and
`google.cloud.storage.__version__` (version string sentinel) on
every call site. Per-call attribute resolution goes through the
`google.cloud.storage` module's attribute table on each call site.
That per-call module-attribute quadruple-read is the workload
measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `Client`, `Bucket`, `Blob`, and
`__version__` per iteration (ITERS scaled so 4 attrs x 20_000 =
~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import google.cloud.storage


_C_BASELINE = google.cloud.storage.Client
_BK_BASELINE = google.cloud.storage.Bucket
_B_BASELINE = google.cloud.storage.Blob
_VERSION_BASELINE = google.cloud.storage.__version__

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = google.cloud.storage.Client
    b = google.cloud.storage.Bucket
    c = google.cloud.storage.Blob
    d = google.cloud.storage.__version__
    if (a is _C_BASELINE
            and b is _BK_BASELINE
            and c is _B_BASELINE
            and d is _VERSION_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"google.cloud.storage module-attribute read acc drift: acc={acc} expected={ITERS}"
print("google_cloud_storage_type_read_hot:", acc)
