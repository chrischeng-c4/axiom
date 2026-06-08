"""Hot-loop bench for `azure.identity.DefaultAzureCredential` /
`azure.identity.ClientSecretCredential` /
`azure.identity.ManagedIdentityCredential` /
`azure.identity.__version__` module-attribute reads (#1506).

End-user scenario: azure-identity-using services re-resolve
`azure.identity.DefaultAzureCredential` (default credential
chain), `azure.identity.ClientSecretCredential` (service
principal credential), `azure.identity.ManagedIdentityCredential`
(managed identity credential), and `azure.identity.__version__`
(version string sentinel) on every call site. Per-call attribute
resolution goes through the `azure.identity` module's attribute
table on each call site. That per-call module-attribute
quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the
`azure.identity` module via Python-side wrappers). Mamba's shim
returns the same identity-stable sentinels directly from a dense
constant table in the `azure.identity` module-attribute resolver,
short-circuiting CPython's module-dict probe chain for read-only
sentinels.

Workload: 20_000 paired reads of `DefaultAzureCredential`,
`ClientSecretCredential`, `ManagedIdentityCredential`, and
`__version__` per iteration (ITERS scaled so 4 attrs x 20_000 =
~80k attr-reads per run, matching the cross-tier 80k attr-read
budget used by the 4-attr 3p perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import azure.identity


_DAC_BASELINE = azure.identity.DefaultAzureCredential
_CSC_BASELINE = azure.identity.ClientSecretCredential
_MIC_BASELINE = azure.identity.ManagedIdentityCredential
_VERSION_BASELINE = azure.identity.__version__

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = azure.identity.DefaultAzureCredential
    b = azure.identity.ClientSecretCredential
    c = azure.identity.ManagedIdentityCredential
    d = azure.identity.__version__
    if (a is _DAC_BASELINE
            and b is _CSC_BASELINE
            and c is _MIC_BASELINE
            and d is _VERSION_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"azure.identity module-attribute read acc drift: acc={acc} expected={ITERS}"
print("azure_identity_type_read_hot:", acc)
