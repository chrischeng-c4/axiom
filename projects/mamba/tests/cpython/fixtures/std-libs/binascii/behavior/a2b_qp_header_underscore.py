# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "a2b_qp_header_underscore"
# subject = "binascii.a2b_qp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_qp: a2b_qp leaves _ literal by default but maps it to space in header mode"""
import binascii

assert binascii.a2b_qp(b"_") == b"_", "underscore literal by default"
assert binascii.a2b_qp(b"_", header=True) == b" ", "underscore is space in header"

print("a2b_qp_header_underscore OK")
