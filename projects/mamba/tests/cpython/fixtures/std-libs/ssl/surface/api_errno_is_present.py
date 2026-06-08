# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_errno_is_present"
# subject = "ssl.errno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.errno: api_errno_is_present (surface)."""
import ssl

assert hasattr(ssl, "errno")
print("api_errno_is_present OK")
