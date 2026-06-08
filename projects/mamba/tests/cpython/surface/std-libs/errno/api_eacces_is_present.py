# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eacces_is_present"
# subject = "errno.EACCES"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EACCES: api_eacces_is_present (surface)."""
import errno

assert hasattr(errno, "EACCES")
print("api_eacces_is_present OK")
