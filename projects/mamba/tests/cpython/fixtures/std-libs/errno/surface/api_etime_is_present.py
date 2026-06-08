# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_etime_is_present"
# subject = "errno.ETIME"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ETIME: api_etime_is_present (surface)."""
import errno

assert hasattr(errno, "ETIME")
print("api_etime_is_present OK")
