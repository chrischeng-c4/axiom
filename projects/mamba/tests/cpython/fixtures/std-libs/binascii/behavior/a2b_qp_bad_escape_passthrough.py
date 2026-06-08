# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "a2b_qp_bad_escape_passthrough"
# subject = "binascii.a2b_qp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_qp: a2b_qp passes a malformed =escape through verbatim"""
import binascii

assert binascii.a2b_qp(b"=AX") == b"=AX", "bad escape passed through"

print("a2b_qp_bad_escape_passthrough OK")
