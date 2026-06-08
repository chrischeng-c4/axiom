# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_op_no_renegotiation_is_present"
# subject = "ssl.OP_NO_RENEGOTIATION"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.OP_NO_RENEGOTIATION: api_op_no_renegotiation_is_present (surface)."""
import ssl

assert hasattr(ssl, "OP_NO_RENEGOTIATION")
print("api_op_no_renegotiation_is_present OK")
