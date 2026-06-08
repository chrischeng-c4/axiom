# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "a85_adobe_framing_round_trip"
# subject = "base64.a85encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.a85encode: a85encode(adobe=True) wraps output in '<~'/'~>' framing and a85decode(adobe=True) round-trips it, including the empty stream '<~~>'"""
import base64

_adobe = base64.a85encode(b"hello", adobe=True)
assert _adobe.startswith(b"<~") and _adobe.endswith(b"~>"), _adobe
assert base64.a85decode(_adobe, adobe=True) == b"hello", "adobe round-trip"
# An empty Adobe stream decodes to empty bytes.
assert base64.a85decode(b"<~~>", adobe=True) == b"", "adobe empty"
print("a85_adobe_framing_round_trip OK")
