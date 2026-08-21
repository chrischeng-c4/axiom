"""Surface contract for third-party azure-storage-blob package.

# type-regime: monomorphic

Probes: azure.storage.blob.BlobServiceClient,
azure.storage.blob.ContainerClient,
azure.storage.blob.BlobClient,
azure.storage.blob.__version__.
CPython 3.12 is the oracle.
"""

import azure.storage.blob  # type: ignore[import]

# Core API
assert hasattr(azure.storage.blob, "BlobServiceClient"), "BlobServiceClient"
assert hasattr(azure.storage.blob, "ContainerClient"), "ContainerClient"
assert hasattr(azure.storage.blob, "BlobClient"), "BlobClient"
assert hasattr(azure.storage.blob, "__version__"), "__version__"
assert hasattr(azure.storage.blob, "BlobType"), "BlobType"
assert hasattr(azure.storage.blob, "ContentSettings"), "ContentSettings"
assert hasattr(azure.storage.blob, "StorageErrorCode"), "StorageErrorCode"

# Version
assert isinstance(azure.storage.blob.__version__, str), \
    f"version type = {type(azure.storage.blob.__version__)!r}"

# Classes are callable
assert callable(azure.storage.blob.BlobServiceClient), \
    "BlobServiceClient callable"
assert callable(azure.storage.blob.ContainerClient), \
    "ContainerClient callable"
assert callable(azure.storage.blob.BlobClient), "BlobClient callable"

# BlobServiceClient has expected methods
assert hasattr(azure.storage.blob.BlobServiceClient, "get_container_client"), \
    "get_container_client"
assert hasattr(azure.storage.blob.BlobServiceClient, "get_blob_client"), \
    "get_blob_client"
assert hasattr(azure.storage.blob.BlobServiceClient, "list_containers"), \
    "list_containers"

# ContainerClient has expected methods
assert hasattr(azure.storage.blob.ContainerClient, "get_blob_client"), \
    "container.get_blob_client"
assert hasattr(azure.storage.blob.ContainerClient, "upload_blob"), \
    "container.upload_blob"
assert hasattr(azure.storage.blob.ContainerClient, "list_blobs"), \
    "container.list_blobs"

# Module attributes stable
_bsc_ref = azure.storage.blob.BlobServiceClient
assert azure.storage.blob.BlobServiceClient is _bsc_ref, "BlobServiceClient stable"
_cc_ref = azure.storage.blob.ContainerClient
assert azure.storage.blob.ContainerClient is _cc_ref, "ContainerClient stable"
_bc_ref = azure.storage.blob.BlobClient
assert azure.storage.blob.BlobClient is _bc_ref, "BlobClient stable"
_v_ref = azure.storage.blob.__version__
assert azure.storage.blob.__version__ is _v_ref, "__version__ stable"

print("surface OK")
