# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_epwroff_is_present"
# subject = "errno.EPWROFF"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EPWROFF: api_epwroff_is_present (surface)."""
import errno

assert hasattr(errno, "EPWROFF")
print("api_epwroff_is_present OK")
