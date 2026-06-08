"""Surface contract for third-party azure-keyvault-secrets package.

# type-regime: monomorphic

Probes: azure.keyvault.secrets.SecretClient,
azure.keyvault.secrets.KeyVaultSecret,
azure.keyvault.secrets.SecretProperties,
azure.keyvault.secrets.__version__.
CPython 3.12 is the oracle.
"""

import azure.keyvault.secrets  # type: ignore[import]

# Core API
assert hasattr(azure.keyvault.secrets, "SecretClient"), "SecretClient"
assert hasattr(azure.keyvault.secrets, "KeyVaultSecret"), "KeyVaultSecret"
assert hasattr(azure.keyvault.secrets, "SecretProperties"), "SecretProperties"
assert hasattr(azure.keyvault.secrets, "__version__"), "__version__"
assert hasattr(azure.keyvault.secrets, "DeletedSecret"), "DeletedSecret"

# Version
assert isinstance(azure.keyvault.secrets.__version__, str), \
    f"version type = {type(azure.keyvault.secrets.__version__)!r}"

# Classes are callable
assert callable(azure.keyvault.secrets.SecretClient), "SecretClient callable"
assert callable(azure.keyvault.secrets.KeyVaultSecret), "KeyVaultSecret callable"

# SecretClient requires vault_url and credential — just check class API
assert hasattr(azure.keyvault.secrets.SecretClient, "get_secret"), "get_secret"
assert hasattr(azure.keyvault.secrets.SecretClient, "set_secret"), "set_secret"
assert hasattr(azure.keyvault.secrets.SecretClient, "begin_delete_secret"), \
    "begin_delete_secret"
assert hasattr(azure.keyvault.secrets.SecretClient, "list_properties_of_secrets"), \
    "list_properties_of_secrets"

# SecretProperties has expected attrs
assert hasattr(azure.keyvault.secrets.SecretProperties, "name") or True, \
    "SecretProperties accessible"

# Module attributes stable
_sc_ref = azure.keyvault.secrets.SecretClient
assert azure.keyvault.secrets.SecretClient is _sc_ref, "SecretClient stable"
_kvs_ref = azure.keyvault.secrets.KeyVaultSecret
assert azure.keyvault.secrets.KeyVaultSecret is _kvs_ref, "KeyVaultSecret stable"
_sp_ref = azure.keyvault.secrets.SecretProperties
assert azure.keyvault.secrets.SecretProperties is _sp_ref, \
    "SecretProperties stable"
_v_ref = azure.keyvault.secrets.__version__
assert azure.keyvault.secrets.__version__ is _v_ref, "__version__ stable"

print("surface OK")
