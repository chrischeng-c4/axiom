"""Behavior contract for third-party googleapis-common-protos package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import google.rpc.status_pb2 as _status_pb2  # type: ignore[import]
import google.rpc.code_pb2 as _code_pb2  # type: ignore[import]
import google.rpc.error_details_pb2 as _error_details_pb2  # type: ignore[import]
import google.rpc  # type: ignore[import]

# Rule 1: Status message construction
_s1 = _status_pb2.Status()
assert _s1.code == 0, f"default code = {_s1.code!r}"
assert _s1.message == "", f"default message = {_s1.message!r}"

# Rule 2: Status with code and message
_s2 = _status_pb2.Status(code=5, message="Not Found")
assert _s2.code == 5, f"code = {_s2.code!r}"
assert _s2.message == "Not Found", f"message = {_s2.message!r}"

# Rule 3: Status serialization to bytes
_s3 = _status_pb2.Status(code=0, message="OK")
_b3 = _s3.SerializeToString()
assert isinstance(_b3, bytes), f"serialized type = {type(_b3)!r}"
_s3b = _status_pb2.Status()
_s3b.ParseFromString(_b3)
assert _s3b.code == 0, f"deserialized code = {_s3b.code!r}"
assert _s3b.message == "OK", f"deserialized message = {_s3b.message!r}"

# Rule 4: BadRequest error details
_br4 = _error_details_pb2.BadRequest()
assert hasattr(_br4, "field_violations"), "field_violations"

# Rule 5: RetryInfo has retry_delay
_ri5 = _error_details_pb2.RetryInfo()
assert hasattr(_ri5, "retry_delay"), "retry_delay"

# Rule 6: Module attributes are identity-stable
_s_ref = google.rpc.status_pb2
_c_ref = google.rpc.code_pb2
_ed_ref = google.rpc.error_details_pb2
for _ in range(5):
    assert google.rpc.status_pb2 is _s_ref, "status_pb2 stable"
    assert google.rpc.code_pb2 is _c_ref, "code_pb2 stable"
    assert google.rpc.error_details_pb2 is _ed_ref, "error_details_pb2 stable"

print("behavior OK")
