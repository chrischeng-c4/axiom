# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_econnreset_is_present"
# subject = "errno.ECONNRESET"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ECONNRESET: api_econnreset_is_present (surface)."""
import errno

assert hasattr(errno, "ECONNRESET")
print("api_econnreset_is_present OK")
