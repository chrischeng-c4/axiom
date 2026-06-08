# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "unhexlify_accepts_str"
# subject = "binascii.unhexlify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.unhexlify: unhexlify and a2b_hex accept ASCII str input, not just bytes"""
import binascii

assert binascii.unhexlify("deadbeef") == b"\xde\xad\xbe\xef", "str unhexlify"
assert binascii.a2b_hex("0102ff") == b"\x01\x02\xff", "a2b_hex str input"

print("unhexlify_accepts_str OK")
