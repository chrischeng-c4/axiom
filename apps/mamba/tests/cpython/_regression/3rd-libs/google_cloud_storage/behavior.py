"""Behavior contract for third-party google-cloud-storage package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import google.cloud.storage  # type: ignore[import]

# Rule 1: Client is callable
_c1 = google.cloud.storage.Client
assert callable(_c1), "Client callable"
assert hasattr(_c1, "bucket"), "client.bucket"
assert hasattr(_c1, "create_bucket"), "client.create_bucket"
assert hasattr(_c1, "list_buckets"), "client.list_buckets"

# Rule 2: Blob has content management methods
_b2 = google.cloud.storage.Blob
assert hasattr(_b2, "upload_from_string"), "upload_from_string"
assert hasattr(_b2, "upload_from_filename"), "upload_from_filename"
assert hasattr(_b2, "download_as_bytes"), "download_as_bytes"
assert hasattr(_b2, "download_as_text"), "download_as_text"
assert hasattr(_b2, "download_to_filename"), "download_to_filename"

# Rule 3: Bucket has blob management methods
_bkt3 = google.cloud.storage.Bucket
assert hasattr(_bkt3, "blob"), "bucket.blob"
assert hasattr(_bkt3, "list_blobs"), "bucket.list_blobs"
assert hasattr(_bkt3, "delete"), "bucket.delete"

# Rule 4: __version__ is dotted version string
_v4 = google.cloud.storage.__version__
_parts4 = _v4.split(".")
assert len(_parts4) >= 2, f"version parts = {_parts4!r}"
assert _parts4[0].isdigit(), f"major numeric = {_parts4[0]!r}"

# Rule 5: Classes are distinct
assert google.cloud.storage.Client is not google.cloud.storage.Bucket, \
    "Client is not Bucket"
assert google.cloud.storage.Bucket is not google.cloud.storage.Blob, \
    "Bucket is not Blob"
assert google.cloud.storage.Client is not google.cloud.storage.Blob, \
    "Client is not Blob"

# Rule 6: Module attributes are identity-stable
_c_ref = google.cloud.storage.Client
_b_ref = google.cloud.storage.Bucket
_bl_ref = google.cloud.storage.Blob
_v_ref = google.cloud.storage.__version__
for _ in range(5):
    assert google.cloud.storage.Client is _c_ref, "Client stable"
    assert google.cloud.storage.Bucket is _b_ref, "Bucket stable"
    assert google.cloud.storage.Blob is _bl_ref, "Blob stable"
    assert google.cloud.storage.__version__ is _v_ref, "__version__ stable"

print("behavior OK")
