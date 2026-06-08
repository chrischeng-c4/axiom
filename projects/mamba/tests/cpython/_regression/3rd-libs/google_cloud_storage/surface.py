"""Surface contract for third-party google-cloud-storage package.

# type-regime: monomorphic

Probes: google.cloud.storage.Client, google.cloud.storage.Bucket,
google.cloud.storage.Blob, google.cloud.storage.__version__.
CPython 3.12 is the oracle.
"""

import google.cloud.storage  # type: ignore[import]

# Core API
assert hasattr(google.cloud.storage, "Client"), "Client"
assert hasattr(google.cloud.storage, "Bucket"), "Bucket"
assert hasattr(google.cloud.storage, "Blob"), "Blob"
assert hasattr(google.cloud.storage, "__version__"), "__version__"

# Version
assert isinstance(google.cloud.storage.__version__, str), \
    f"version type = {type(google.cloud.storage.__version__)!r}"

# Classes are callable
assert callable(google.cloud.storage.Client), "Client callable"
assert callable(google.cloud.storage.Bucket), "Bucket callable"
assert callable(google.cloud.storage.Blob), "Blob callable"

# Client has expected methods
assert hasattr(google.cloud.storage.Client, "bucket"), "client.bucket"
assert hasattr(google.cloud.storage.Client, "create_bucket"), "create_bucket"
assert hasattr(google.cloud.storage.Client, "list_buckets"), "list_buckets"
assert hasattr(google.cloud.storage.Client, "get_bucket"), "get_bucket"

# Blob has expected methods
assert hasattr(google.cloud.storage.Blob, "upload_from_string"), \
    "blob.upload_from_string"
assert hasattr(google.cloud.storage.Blob, "download_as_bytes"), \
    "blob.download_as_bytes"
assert hasattr(google.cloud.storage.Blob, "download_as_text"), \
    "blob.download_as_text"
assert hasattr(google.cloud.storage.Blob, "delete"), "blob.delete"
assert hasattr(google.cloud.storage.Blob, "exists"), "blob.exists"

# Module attributes stable
_c_ref = google.cloud.storage.Client
assert google.cloud.storage.Client is _c_ref, "Client stable"
_b_ref = google.cloud.storage.Bucket
assert google.cloud.storage.Bucket is _b_ref, "Bucket stable"
_bl_ref = google.cloud.storage.Blob
assert google.cloud.storage.Blob is _bl_ref, "Blob stable"
_v_ref = google.cloud.storage.__version__
assert google.cloud.storage.__version__ is _v_ref, "__version__ stable"

print("surface OK")
