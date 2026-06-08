# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eaddrinuse_is_present"
# subject = "errno.EADDRINUSE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EADDRINUSE: api_eaddrinuse_is_present (surface)."""
import errno

assert hasattr(errno, "EADDRINUSE")
print("api_eaddrinuse_is_present OK")
