# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_ebusy_is_present"
# subject = "errno.EBUSY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EBUSY: api_ebusy_is_present (surface)."""
import errno

assert hasattr(errno, "EBUSY")
print("api_ebusy_is_present OK")
