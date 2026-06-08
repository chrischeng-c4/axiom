# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "a2b_uu_marker_length"
# subject = "binascii.a2b_uu"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_uu: a2b_uu derives output length from the leading marker byte; blank lines decode empty"""
import binascii

# a2b_uu derives output length only from the leading marker byte; trailing
# garbage is ignored, so high marker bytes yield runs of NUL.
assert binascii.a2b_uu(b"\x7f") == b"\x00" * 31, "marker 0x7f -> 31 NUL"
assert binascii.a2b_uu(b"\x80") == b"\x00" * 32, "marker 0x80 -> 32 NUL"
# Empty/blank lines decode to empty bytes.
assert binascii.a2b_uu(b" \n") == b"", "blank line decodes empty"
assert binascii.a2b_uu(b"`\n") == b"", "backtick line decodes empty"

print("a2b_uu_marker_length OK")
