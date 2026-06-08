# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_qp_escapes_reserved"
# subject = "binascii.b2a_qp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_qp: b2a_qp escapes non-printable/reserved bytes as =XX uppercase hex"""
import binascii

assert binascii.b2a_qp(b"\x7f") == b"=7F", "DEL escaped"
assert binascii.b2a_qp(b"=") == b"=3D", "literal = escaped"
assert binascii.b2a_qp(b" ") == b"=20", "lone trailing space escaped"
assert binascii.b2a_qp(b".") == b"=2E", "leading dot escaped"

print("b2a_qp_escapes_reserved OK")
