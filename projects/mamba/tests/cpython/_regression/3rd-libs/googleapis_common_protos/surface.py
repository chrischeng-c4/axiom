"""Surface contract for third-party googleapis-common-protos package.

# type-regime: monomorphic

Probes: google.rpc.status_pb2, google.rpc.code_pb2,
google.rpc.error_details_pb2.
CPython 3.12 is the oracle.
"""

import google.rpc.status_pb2 as _status_pb2  # type: ignore[import]
import google.rpc.code_pb2 as _code_pb2  # type: ignore[import]
import google.rpc.error_details_pb2 as _error_details_pb2  # type: ignore[import]
import google.rpc  # type: ignore[import]

# Submodules accessible from google.rpc
assert hasattr(google.rpc, "status_pb2"), "status_pb2"
assert hasattr(google.rpc, "code_pb2"), "code_pb2"
assert hasattr(google.rpc, "error_details_pb2"), "error_details_pb2"

# status_pb2: Status message
assert hasattr(_status_pb2, "Status"), "Status"
assert callable(_status_pb2.Status), "Status callable"
_st = _status_pb2.Status()
assert hasattr(_st, "code"), "Status.code"
assert hasattr(_st, "message"), "Status.message"
assert hasattr(_st, "details"), "Status.details"

# code_pb2: canonical error codes
assert hasattr(_code_pb2, "Code"), "Code"
# Common codes
assert hasattr(_code_pb2, "OK") or hasattr(_code_pb2.Code, "OK"), "OK code"

# error_details_pb2: extended error details
assert hasattr(_error_details_pb2, "BadRequest"), "BadRequest"
assert hasattr(_error_details_pb2, "RetryInfo"), "RetryInfo"
assert hasattr(_error_details_pb2, "ErrorInfo"), "ErrorInfo"

# Module attributes stable
_s_ref = google.rpc.status_pb2
assert google.rpc.status_pb2 is _s_ref, "status_pb2 stable"
_c_ref = google.rpc.code_pb2
assert google.rpc.code_pb2 is _c_ref, "code_pb2 stable"
_ed_ref = google.rpc.error_details_pb2
assert google.rpc.error_details_pb2 is _ed_ref, "error_details_pb2 stable"

print("surface OK")
