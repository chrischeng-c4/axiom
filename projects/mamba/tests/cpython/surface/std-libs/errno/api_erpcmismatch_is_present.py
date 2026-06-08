# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_erpcmismatch_is_present"
# subject = "errno.ERPCMISMATCH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ERPCMISMATCH: api_erpcmismatch_is_present (surface)."""
import errno

assert hasattr(errno, "ERPCMISMATCH")
print("api_erpcmismatch_is_present OK")
