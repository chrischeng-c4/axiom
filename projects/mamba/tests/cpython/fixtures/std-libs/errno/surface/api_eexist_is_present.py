# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eexist_is_present"
# subject = "errno.EEXIST"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EEXIST: api_eexist_is_present (surface)."""
import errno

assert hasattr(errno, "EEXIST")
print("api_eexist_is_present OK")
