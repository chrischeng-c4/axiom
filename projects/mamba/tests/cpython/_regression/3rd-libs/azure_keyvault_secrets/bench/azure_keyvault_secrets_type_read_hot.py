"""Hot-loop bench for `azure.keyvault.secrets.SecretClient` /
`azure.keyvault.secrets.KeyVaultSecret` /
`azure.keyvault.secrets.SecretProperties` /
`azure.keyvault.secrets.__version__` module-attribute reads (#1508).

End-user scenario: azure-keyvault-secrets-using services re-resolve
`azure.keyvault.secrets.SecretClient` (client class),
`azure.keyvault.secrets.KeyVaultSecret` (secret model),
`azure.keyvault.secrets.SecretProperties` (secret properties model),
and `azure.keyvault.secrets.__version__` (version string sentinel)
on every call site. Per-call attribute resolution goes through the
`azure.keyvault.secrets` module's attribute table on each call site.
That per-call module-attribute quadruple-read is the workload
measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `SecretClient`, `KeyVaultSecret`,
`SecretProperties`, and `__version__` per iteration (ITERS scaled
so 4 attrs x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import azure.keyvault.secrets


_SC_BASELINE = azure.keyvault.secrets.SecretClient
_KS_BASELINE = azure.keyvault.secrets.KeyVaultSecret
_SP_BASELINE = azure.keyvault.secrets.SecretProperties
_VERSION_BASELINE = azure.keyvault.secrets.__version__

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = azure.keyvault.secrets.SecretClient
    b = azure.keyvault.secrets.KeyVaultSecret
    c = azure.keyvault.secrets.SecretProperties
    d = azure.keyvault.secrets.__version__
    if (a is _SC_BASELINE
            and b is _KS_BASELINE
            and c is _SP_BASELINE
            and d is _VERSION_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"azure.keyvault.secrets module-attribute read acc drift: acc={acc} expected={ITERS}"
print("azure_keyvault_secrets_type_read_hot:", acc)
