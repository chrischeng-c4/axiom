# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_op_single_ecdh_use_is_present"
# subject = "ssl.OP_SINGLE_ECDH_USE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.OP_SINGLE_ECDH_USE: api_op_single_ecdh_use_is_present (surface)."""
import ssl

assert hasattr(ssl, "OP_SINGLE_ECDH_USE")
print("api_op_single_ecdh_use_is_present OK")
