"""Behavior contract for third-party azure-identity package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import azure.identity  # type: ignore[import]

# Rule 1: ClientSecretCredential stores tenant_id, client_id
_csc1 = azure.identity.ClientSecretCredential(
    tenant_id="my-tenant",
    client_id="my-client",
    client_secret="my-secret",
)
assert hasattr(_csc1, "get_token"), "get_token"
assert callable(_csc1.get_token), "get_token callable"

# Rule 2: EnvironmentCredential has get_token
_ec2 = azure.identity.EnvironmentCredential()
assert hasattr(_ec2, "get_token"), "env.get_token"
assert callable(_ec2.get_token), "env.get_token callable"

# Rule 3: ManagedIdentityCredential has get_token
_mic3 = azure.identity.ManagedIdentityCredential()
assert hasattr(_mic3, "get_token"), "mic.get_token"
assert callable(_mic3.get_token), "mic.get_token callable"

# Rule 4: ChainedTokenCredential accepts credential list
_ctc4 = azure.identity.ChainedTokenCredential(
    azure.identity.EnvironmentCredential(),
    azure.identity.ManagedIdentityCredential(),
)
assert hasattr(_ctc4, "get_token"), "ctc.get_token"
assert callable(_ctc4.get_token), "ctc.get_token callable"

# Rule 5: __version__ format
_v5 = azure.identity.__version__
_parts5 = _v5.split(".")
assert len(_parts5) >= 2, f"version parts = {_parts5!r}"

# Rule 6: Module attributes are identity-stable
_dac_ref = azure.identity.DefaultAzureCredential
_csc_ref = azure.identity.ClientSecretCredential
_mic_ref = azure.identity.ManagedIdentityCredential
_v_ref = azure.identity.__version__
for _ in range(5):
    assert azure.identity.DefaultAzureCredential is _dac_ref, \
        "DefaultAzureCredential stable"
    assert azure.identity.ClientSecretCredential is _csc_ref, \
        "ClientSecretCredential stable"
    assert azure.identity.ManagedIdentityCredential is _mic_ref, \
        "ManagedIdentityCredential stable"
    assert azure.identity.__version__ is _v_ref, "__version__ stable"

print("behavior OK")
