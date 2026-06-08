"""Behavior contract for third-party protobuf package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import google.protobuf.message  # type: ignore[import]
import google.protobuf.json_format  # type: ignore[import]
import google.protobuf.text_format  # type: ignore[import]
import google.protobuf  # type: ignore[import]

# Rule 1: Message base class is abstract (cannot instantiate directly)
_m1 = google.protobuf.message.Message
assert callable(_m1), "Message callable"
assert hasattr(_m1, "SerializeToString"), "SerializeToString"
assert hasattr(_m1, "ParseFromString"), "ParseFromString"
assert hasattr(_m1, "MergeFromString"), "MergeFromString"
assert hasattr(_m1, "ByteSize"), "ByteSize"

# Rule 2: DecodeError is an exception
_de2 = google.protobuf.message.DecodeError
assert issubclass(_de2, Exception), "DecodeError < Exception"

# Rule 3: EncodeError is an exception
_ee3 = google.protobuf.message.EncodeError
assert issubclass(_ee3, Exception), "EncodeError < Exception"

# Rule 4: json_format.MessageToJson is callable
assert callable(google.protobuf.json_format.MessageToJson), "MessageToJson"
assert callable(google.protobuf.json_format.MessageToDict), "MessageToDict"
assert callable(google.protobuf.json_format.Parse), "Parse"
assert callable(google.protobuf.json_format.ParseDict), "ParseDict"

# Rule 5: text_format.MessageToString is callable
assert callable(google.protobuf.text_format.MessageToString), "MessageToString"
assert callable(google.protobuf.text_format.Parse), "text_format.Parse"

# Rule 6: Module attributes are identity-stable
_m_ref = google.protobuf.message
_d_ref = google.protobuf.descriptor
_jf_ref = google.protobuf.json_format
_tf_ref = google.protobuf.text_format
for _ in range(5):
    assert google.protobuf.message is _m_ref, "message stable"
    assert google.protobuf.descriptor is _d_ref, "descriptor stable"
    assert google.protobuf.json_format is _jf_ref, "json_format stable"
    assert google.protobuf.text_format is _tf_ref, "text_format stable"

print("behavior OK")
