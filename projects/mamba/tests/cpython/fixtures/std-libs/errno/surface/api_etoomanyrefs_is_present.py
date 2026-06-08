# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_etoomanyrefs_is_present"
# subject = "errno.ETOOMANYREFS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ETOOMANYREFS: api_etoomanyrefs_is_present (surface)."""
import errno

assert hasattr(errno, "ETOOMANYREFS")
print("api_etoomanyrefs_is_present OK")
