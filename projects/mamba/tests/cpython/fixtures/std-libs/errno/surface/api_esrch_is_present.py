# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_esrch_is_present"
# subject = "errno.ESRCH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ESRCH: api_esrch_is_present (surface)."""
import errno

assert hasattr(errno, "ESRCH")
print("api_esrch_is_present OK")
