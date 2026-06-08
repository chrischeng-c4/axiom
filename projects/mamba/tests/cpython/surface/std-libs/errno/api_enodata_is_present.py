# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enodata_is_present"
# subject = "errno.ENODATA"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENODATA: api_enodata_is_present (surface)."""
import errno

assert hasattr(errno, "ENODATA")
print("api_enodata_is_present OK")
