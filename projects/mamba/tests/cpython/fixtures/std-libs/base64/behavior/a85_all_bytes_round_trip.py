# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "a85_all_bytes_round_trip"
# subject = "base64.a85encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.a85encode: a85encode then a85decode round-trips every byte value 0..255 unchanged"""
import base64

_payload = bytes(range(256))
_enc = base64.a85encode(_payload)
assert isinstance(_enc, bytes), type(_enc)
assert base64.a85decode(_enc) == _payload, "a85 round-trip"
print("a85_all_bytes_round_trip OK")
