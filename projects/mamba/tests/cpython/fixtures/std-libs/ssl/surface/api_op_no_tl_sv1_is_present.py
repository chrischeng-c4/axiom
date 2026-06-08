# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_op_no_tl_sv1_is_present"
# subject = "ssl.OP_NO_TLSv1"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.OP_NO_TLSv1: api_op_no_tl_sv1_is_present (surface)."""
import ssl

assert hasattr(ssl, "OP_NO_TLSv1")
print("api_op_no_tl_sv1_is_present OK")
