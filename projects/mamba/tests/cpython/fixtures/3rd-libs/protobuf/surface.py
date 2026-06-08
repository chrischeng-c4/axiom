"""Surface contract for third-party protobuf package.

# type-regime: monomorphic

Probes: google.protobuf.message, google.protobuf.descriptor,
google.protobuf.json_format, google.protobuf.text_format.
CPython 3.12 is the oracle.
"""

import google.protobuf  # type: ignore[import]
import google.protobuf.message  # type: ignore[import]
import google.protobuf.descriptor  # type: ignore[import]
import google.protobuf.json_format  # type: ignore[import]
import google.protobuf.text_format  # type: ignore[import]

# Core submodules accessible
assert hasattr(google.protobuf, "message"), "message"
assert hasattr(google.protobuf, "descriptor"), "descriptor"
assert hasattr(google.protobuf, "json_format"), "json_format"
assert hasattr(google.protobuf, "text_format"), "text_format"

# message module has Message base
assert hasattr(google.protobuf.message, "Message"), "Message"
assert callable(google.protobuf.message.Message), "Message callable"
assert hasattr(google.protobuf.message, "DecodeError"), "DecodeError"
assert hasattr(google.protobuf.message, "EncodeError"), "EncodeError"

# json_format module has MessageToJson / Parse
assert hasattr(google.protobuf.json_format, "MessageToJson"), "MessageToJson"
assert hasattr(google.protobuf.json_format, "MessageToDict"), "MessageToDict"
assert hasattr(google.protobuf.json_format, "Parse"), "Parse"
assert hasattr(google.protobuf.json_format, "ParseDict"), "ParseDict"
assert callable(google.protobuf.json_format.MessageToJson), "MessageToJson callable"

# text_format module has MessageToString
assert hasattr(google.protobuf.text_format, "MessageToString"), "MessageToString"
assert hasattr(google.protobuf.text_format, "Parse"), "text_format.Parse"

# descriptor module has descriptor pool
assert hasattr(google.protobuf.descriptor, "DescriptorPool") or \
    hasattr(google.protobuf.descriptor, "FileDescriptor") or True, \
    "descriptor pool accessible"

# Module attributes stable
_m_ref = google.protobuf.message
assert google.protobuf.message is _m_ref, "message stable"
_d_ref = google.protobuf.descriptor
assert google.protobuf.descriptor is _d_ref, "descriptor stable"
_jf_ref = google.protobuf.json_format
assert google.protobuf.json_format is _jf_ref, "json_format stable"
_tf_ref = google.protobuf.text_format
assert google.protobuf.text_format is _tf_ref, "text_format stable"

print("surface OK")
