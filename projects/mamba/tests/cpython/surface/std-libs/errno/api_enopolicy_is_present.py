# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enopolicy_is_present"
# subject = "errno.ENOPOLICY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOPOLICY: api_enopolicy_is_present (surface)."""
import errno

assert hasattr(errno, "ENOPOLICY")
print("api_enopolicy_is_present OK")
