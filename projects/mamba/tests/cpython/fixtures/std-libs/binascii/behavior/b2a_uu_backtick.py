# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_uu_backtick"
# subject = "binascii.b2a_uu"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_uu: b2a_uu(backtick=True) writes grave accents for zeros; both forms decode equal"""
import binascii

assert binascii.b2a_uu(b"", backtick=True) == b"`\n", "b2a_uu(empty, backtick)"
assert binascii.b2a_uu(b"\x00Cat", backtick=True) == b"$`$-A=```\n", "backtick encoding"
# Both space- and backtick-padded forms decode to the same bytes.
assert binascii.a2b_uu(b"$`$-A=```\n") == binascii.a2b_uu(b"$ $-A=   \n"), "backtick decode"

print("b2a_uu_backtick OK")
