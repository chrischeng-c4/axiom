# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "b2a_qp_is_callable"
# subject = "binascii.b2a_qp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""binascii.b2a_qp: b2a_qp_is_callable (surface)."""
import binascii

assert callable(binascii.b2a_qp)
print("b2a_qp_is_callable OK")
