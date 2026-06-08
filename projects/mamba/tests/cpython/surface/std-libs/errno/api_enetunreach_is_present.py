# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enetunreach_is_present"
# subject = "errno.ENETUNREACH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENETUNREACH: api_enetunreach_is_present (surface)."""
import errno

assert hasattr(errno, "ENETUNREACH")
print("api_enetunreach_is_present OK")
