# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "hex_aliases_match"
# subject = "binascii.b2a_hex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_hex: b2a_hex aliases hexlify and a2b_hex aliases unhexlify"""
import binascii

_data = b"hello world"
assert binascii.b2a_hex(_data) == binascii.hexlify(_data), "b2a_hex == hexlify"
assert binascii.a2b_hex(b"0102ff") == binascii.unhexlify(b"0102ff"), "a2b_hex == unhexlify"

print("hex_aliases_match OK")
