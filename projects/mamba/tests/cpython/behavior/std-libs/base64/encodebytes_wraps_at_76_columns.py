# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "encodebytes_wraps_at_76_columns"
# subject = "base64.encodebytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.encodebytes: encodebytes inserts newlines so every line is at most 76 chars, and decodebytes round-trips the wrapped output back to the original"""
import base64

_long = b"x" * 100
_eb = base64.encodebytes(_long)
assert b"\n" in _eb, "encodebytes wraps with newlines"
for _line in _eb.split(b"\n"):
    assert len(_line) <= 76, len(_line)
assert base64.decodebytes(_eb) == _long, "decodebytes round-trip"
print("encodebytes_wraps_at_76_columns OK")
