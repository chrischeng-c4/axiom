"""Behavior contract for third-party azure-keyvault-secrets package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import azure.keyvault.secrets  # type: ignore[import]

# Rule 1: SecretClient class API
_sc1 = azure.keyvault.secrets.SecretClient
assert callable(_sc1), "SecretClient callable"
assert hasattr(_sc1, "get_secret"), "get_secret method"
assert hasattr(_sc1, "set_secret"), "set_secret method"
assert hasattr(_sc1, "begin_delete_secret"), "begin_delete_secret method"
assert hasattr(_sc1, "list_properties_of_secrets"), "list_properties_of_secrets"

# Rule 2: KeyVaultSecret class is callable
_kvs2 = azure.keyvault.secrets.KeyVaultSecret
assert callable(_kvs2), "KeyVaultSecret callable"

# Rule 3: SecretProperties class is callable
_sp3 = azure.keyvault.secrets.SecretProperties
assert callable(_sp3), "SecretProperties callable"

# Rule 4: DeletedSecret is a class
_ds4 = azure.keyvault.secrets.DeletedSecret
assert callable(_ds4), "DeletedSecret callable"

# Rule 5: __version__ is a dotted version string
_v5 = azure.keyvault.secrets.__version__
_parts5 = _v5.split(".")
assert len(_parts5) >= 2, f"version parts = {_parts5!r}"
assert _parts5[0].isdigit(), f"major version numeric = {_parts5[0]!r}"

# Rule 6: Module attributes are identity-stable
_sc_ref = azure.keyvault.secrets.SecretClient
_kvs_ref = azure.keyvault.secrets.KeyVaultSecret
_sp_ref = azure.keyvault.secrets.SecretProperties
_v_ref = azure.keyvault.secrets.__version__
for _ in range(5):
    assert azure.keyvault.secrets.SecretClient is _sc_ref, "SecretClient stable"
    assert azure.keyvault.secrets.KeyVaultSecret is _kvs_ref, \
        "KeyVaultSecret stable"
    assert azure.keyvault.secrets.SecretProperties is _sp_ref, \
        "SecretProperties stable"
    assert azure.keyvault.secrets.__version__ is _v_ref, "__version__ stable"

print("behavior OK")
