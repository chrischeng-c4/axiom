# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b85_all_bytes_round_trip"
# subject = "base64.b85encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b85encode: b85encode then b85decode round-trips every byte value 0..255 unchanged, and empty input round-trips to empty bytes"""
import base64

_payload = bytes(range(256))
_enc = base64.b85encode(_payload)
assert isinstance(_enc, bytes), type(_enc)
assert base64.b85decode(_enc) == _payload, "b85 round-trip"
# Empty input round-trips to empty bytes.
assert base64.b85encode(b"") == b"", "b85 empty encode"
assert base64.b85decode(b"") == b"", "b85 empty decode"
print("b85_all_bytes_round_trip OK")
