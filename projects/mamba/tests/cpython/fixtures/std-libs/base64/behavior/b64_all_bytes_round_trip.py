# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64_all_bytes_round_trip"
# subject = "base64.b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64encode: b64encode then b64decode round-trips every byte value 0..255 unchanged"""
import base64

_all_bytes = bytes(range(256))
_enc = base64.b64encode(_all_bytes)
assert base64.b64decode(_enc) == _all_bytes, "all bytes round-trip"
print("b64_all_bytes_round_trip OK")
