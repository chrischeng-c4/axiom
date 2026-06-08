# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_op_enable_middlebox_compat_is_present"
# subject = "ssl.OP_ENABLE_MIDDLEBOX_COMPAT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.OP_ENABLE_MIDDLEBOX_COMPAT: api_op_enable_middlebox_compat_is_present (surface)."""
import ssl

assert hasattr(ssl, "OP_ENABLE_MIDDLEBOX_COMPAT")
print("api_op_enable_middlebox_compat_is_present OK")
