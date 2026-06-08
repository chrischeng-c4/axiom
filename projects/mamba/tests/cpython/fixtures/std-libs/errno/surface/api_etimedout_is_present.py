# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_etimedout_is_present"
# subject = "errno.ETIMEDOUT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ETIMEDOUT: api_etimedout_is_present (surface)."""
import errno

assert hasattr(errno, "ETIMEDOUT")
print("api_etimedout_is_present OK")
