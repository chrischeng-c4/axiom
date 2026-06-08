# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_edeverr_is_present"
# subject = "errno.EDEVERR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EDEVERR: api_edeverr_is_present (surface)."""
import errno

assert hasattr(errno, "EDEVERR")
print("api_edeverr_is_present OK")
