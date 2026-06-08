"""Surface contract for third-party azure-identity package.

# type-regime: monomorphic

Probes: azure.identity.DefaultAzureCredential,
azure.identity.ClientSecretCredential,
azure.identity.ManagedIdentityCredential,
azure.identity.__version__.
CPython 3.12 is the oracle.
"""

import azure.identity  # type: ignore[import]

# Core API
assert hasattr(azure.identity, "DefaultAzureCredential"), \
    "DefaultAzureCredential"
assert hasattr(azure.identity, "ClientSecretCredential"), \
    "ClientSecretCredential"
assert hasattr(azure.identity, "ManagedIdentityCredential"), \
    "ManagedIdentityCredential"
assert hasattr(azure.identity, "__version__"), "__version__"
assert hasattr(azure.identity, "EnvironmentCredential"), "EnvironmentCredential"
assert hasattr(azure.identity, "CertificateCredential"), "CertificateCredential"
assert hasattr(azure.identity, "ChainedTokenCredential"), "ChainedTokenCredential"

# Version
assert isinstance(azure.identity.__version__, str), \
    f"version type = {type(azure.identity.__version__)!r}"

# Classes are callable
assert callable(azure.identity.DefaultAzureCredential), \
    "DefaultAzureCredential callable"
assert callable(azure.identity.ClientSecretCredential), \
    "ClientSecretCredential callable"
assert callable(azure.identity.ManagedIdentityCredential), \
    "ManagedIdentityCredential callable"

# ClientSecretCredential has get_token method
_csc = azure.identity.ClientSecretCredential(
    tenant_id="tenant-id",
    client_id="client-id",
    client_secret="secret",
)
assert hasattr(_csc, "get_token"), "csc.get_token"
assert callable(_csc.get_token), "csc.get_token callable"

# Module attributes stable
_dac_ref = azure.identity.DefaultAzureCredential
assert azure.identity.DefaultAzureCredential is _dac_ref, \
    "DefaultAzureCredential stable"
_csc_ref = azure.identity.ClientSecretCredential
assert azure.identity.ClientSecretCredential is _csc_ref, \
    "ClientSecretCredential stable"
_mic_ref = azure.identity.ManagedIdentityCredential
assert azure.identity.ManagedIdentityCredential is _mic_ref, \
    "ManagedIdentityCredential stable"
_v_ref = azure.identity.__version__
assert azure.identity.__version__ is _v_ref, "__version__ stable"

print("surface OK")
