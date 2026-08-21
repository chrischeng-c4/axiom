# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "a2b_qp_is_callable"
# subject = "binascii.a2b_qp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""binascii.a2b_qp: a2b_qp_is_callable (surface)."""
import binascii

assert callable(binascii.a2b_qp)
print("a2b_qp_is_callable OK")
