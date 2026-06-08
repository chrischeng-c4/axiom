# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_qp_crlf_normalisation"
# subject = "binascii.b2a_qp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_qp: b2a_qp normalises bare CR/LF to CRLF and escapes high bytes"""
import binascii

assert binascii.b2a_qp(b"\xff\r\n\xff\n\xff") == b"=FF\r\n=FF\r\n=FF", "CRLF + =FF"

print("b2a_qp_crlf_normalisation OK")
