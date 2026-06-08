# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eqfull_is_present"
# subject = "errno.EQFULL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EQFULL: api_eqfull_is_present (surface)."""
import errno

assert hasattr(errno, "EQFULL")
print("api_eqfull_is_present OK")
