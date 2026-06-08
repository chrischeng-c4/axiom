# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "surface"
# case = "api_b2a_qp_is_present"
# subject = "binascii.b2a_qp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""binascii.b2a_qp: api_b2a_qp_is_present (surface)."""
import binascii

assert hasattr(binascii, "b2a_qp")
print("api_b2a_qp_is_present OK")
