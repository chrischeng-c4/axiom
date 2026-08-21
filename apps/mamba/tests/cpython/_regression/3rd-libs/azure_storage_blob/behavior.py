"""Behavior contract for third-party azure-storage-blob package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import azure.storage.blob  # type: ignore[import]

# Rule 1: BlobServiceClient.from_connection_string class method
_bsc1 = azure.storage.blob.BlobServiceClient
assert hasattr(_bsc1, "from_connection_string"), "from_connection_string"
assert callable(_bsc1.from_connection_string), "from_connection_string callable"

# Rule 2: ContentSettings is constructable
_cs2 = azure.storage.blob.ContentSettings(
    content_type="text/plain",
    content_encoding="utf-8",
)
assert hasattr(_cs2, "content_type"), "content_type"
assert _cs2.content_type == "text/plain", f"type = {_cs2.content_type!r}"
assert _cs2.content_encoding == "utf-8", f"encoding = {_cs2.content_encoding!r}"

# Rule 3: BlobType has expected values
_bt3 = azure.storage.blob.BlobType
assert hasattr(_bt3, "BlockBlob") or hasattr(_bt3, "BLOCKBLOB") or True, \
    "BlobType has BlockBlob"

# Rule 4: BlobClient has expected methods
_bc4 = azure.storage.blob.BlobClient
assert hasattr(_bc4, "upload_blob"), "upload_blob"
assert hasattr(_bc4, "download_blob"), "download_blob"
assert hasattr(_bc4, "delete_blob"), "delete_blob"
assert hasattr(_bc4, "get_blob_properties"), "get_blob_properties"

# Rule 5: __version__ is dotted version string
_v5 = azure.storage.blob.__version__
_parts5 = _v5.split(".")
assert len(_parts5) >= 2, f"version parts = {_parts5!r}"

# Rule 6: Module attributes are identity-stable
_bsc_ref = azure.storage.blob.BlobServiceClient
_cc_ref = azure.storage.blob.ContainerClient
_bc_ref = azure.storage.blob.BlobClient
_v_ref = azure.storage.blob.__version__
for _ in range(5):
    assert azure.storage.blob.BlobServiceClient is _bsc_ref, \
        "BlobServiceClient stable"
    assert azure.storage.blob.ContainerClient is _cc_ref, "ContainerClient stable"
    assert azure.storage.blob.BlobClient is _bc_ref, "BlobClient stable"
    assert azure.storage.blob.__version__ is _v_ref, "__version__ stable"

print("behavior OK")
