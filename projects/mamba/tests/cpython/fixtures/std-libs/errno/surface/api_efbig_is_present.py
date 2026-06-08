# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_efbig_is_present"
# subject = "errno.EFBIG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EFBIG: api_efbig_is_present (surface)."""
import errno

assert hasattr(errno, "EFBIG")
print("api_efbig_is_present OK")
