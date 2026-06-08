# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "codecs_accept_empty_input"
# subject = "binascii.hexlify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.hexlify: every b2a_*/a2b_* codec accepts empty input without raising"""
import binascii

for _name in ("b2a_base64", "b2a_hex", "b2a_qp", "b2a_uu", "hexlify",
              "a2b_base64", "a2b_hex", "a2b_qp", "a2b_uu", "unhexlify"):
    getattr(binascii, _name)(b"")
assert binascii.crc_hqx(b"", 0) == 0, "crc_hqx empty"
assert binascii.crc32(b"") == 0, "crc32 empty"

print("codecs_accept_empty_input OK")
