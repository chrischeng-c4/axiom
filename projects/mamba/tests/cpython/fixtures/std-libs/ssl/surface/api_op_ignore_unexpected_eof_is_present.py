# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_op_ignore_unexpected_eof_is_present"
# subject = "ssl.OP_IGNORE_UNEXPECTED_EOF"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.OP_IGNORE_UNEXPECTED_EOF: api_op_ignore_unexpected_eof_is_present (surface)."""
import ssl

assert hasattr(ssl, "OP_IGNORE_UNEXPECTED_EOF")
print("api_op_ignore_unexpected_eof_is_present OK")
