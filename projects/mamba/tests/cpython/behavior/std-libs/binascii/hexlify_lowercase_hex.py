# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "hexlify_lowercase_hex"
# subject = "binascii.hexlify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.hexlify: hexlify emits lowercase hex digits and is bytes-typed"""
import binascii

_h = binascii.hexlify(b"\xab\xcd\xef")
assert isinstance(_h, bytes), f"hexlify type = {type(_h)!r}"
assert _h == b"abcdef", f"lowercase hex = {_h!r}"
print("hexlify_lowercase_hex OK")
