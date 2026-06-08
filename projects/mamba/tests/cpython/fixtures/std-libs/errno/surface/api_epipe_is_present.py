# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_epipe_is_present"
# subject = "errno.EPIPE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EPIPE: api_epipe_is_present (surface)."""
import errno

assert hasattr(errno, "EPIPE")
print("api_epipe_is_present OK")
