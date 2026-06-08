"""Hot-loop bench for `azure.storage.blob.BlobServiceClient` /
`azure.storage.blob.ContainerClient` /
`azure.storage.blob.BlobClient` /
`azure.storage.blob.__version__` module-attribute reads (#1507).

End-user scenario: azure-storage-blob-using services re-resolve
`azure.storage.blob.BlobServiceClient` (service-level client),
`azure.storage.blob.ContainerClient` (container-level client),
`azure.storage.blob.BlobClient` (blob-level client), and
`azure.storage.blob.__version__` (version string sentinel) on
every call site. Per-call attribute resolution goes through the
`azure.storage.blob` module's attribute table on each call site.
That per-call module-attribute quadruple-read is the workload
measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `BlobServiceClient`,
`ContainerClient`, `BlobClient`, and `__version__` per iteration
(ITERS scaled so 4 attrs x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import azure.storage.blob


_BSC_BASELINE = azure.storage.blob.BlobServiceClient
_CC_BASELINE = azure.storage.blob.ContainerClient
_BC_BASELINE = azure.storage.blob.BlobClient
_VERSION_BASELINE = azure.storage.blob.__version__

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = azure.storage.blob.BlobServiceClient
    b = azure.storage.blob.ContainerClient
    c = azure.storage.blob.BlobClient
    d = azure.storage.blob.__version__
    if (a is _BSC_BASELINE
            and b is _CC_BASELINE
            and c is _BC_BASELINE
            and d is _VERSION_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"azure.storage.blob module-attribute read acc drift: acc={acc} expected={ITERS}"
print("azure_storage_blob_type_read_hot:", acc)
