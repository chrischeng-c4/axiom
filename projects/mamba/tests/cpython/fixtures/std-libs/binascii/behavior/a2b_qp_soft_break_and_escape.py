# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "a2b_qp_soft_break_and_escape"
# subject = "binascii.a2b_qp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_qp: a2b_qp: lone = is a soft break, == is literal =, =XX is case-insensitive hex"""
import binascii

# A lone '=' is a soft break and disappears; '==' decodes to a literal '='.
assert binascii.a2b_qp(b"=") == b"", "trailing = is soft break"
assert binascii.a2b_qp(b"==") == b"=", "== decodes to ="
# '=XX' is a hex escape (case-insensitive); a soft break joins the next line.
assert binascii.a2b_qp(b"=AB") == b"\xab", "=AB hex escape"
assert binascii.a2b_qp(b"=ab") == b"\xab", "lowercase hex escape"
assert binascii.a2b_qp(b"=\nAB") == b"AB", "soft break before text"

print("a2b_qp_soft_break_and_escape OK")
