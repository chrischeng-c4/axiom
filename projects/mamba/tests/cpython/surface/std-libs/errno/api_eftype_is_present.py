# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eftype_is_present"
# subject = "errno.EFTYPE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EFTYPE: api_eftype_is_present (surface)."""
import errno

assert hasattr(errno, "EFTYPE")
print("api_eftype_is_present OK")
