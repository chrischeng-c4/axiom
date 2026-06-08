# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enetreset_is_present"
# subject = "errno.ENETRESET"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENETRESET: api_enetreset_is_present (surface)."""
import errno

assert hasattr(errno, "ENETRESET")
print("api_enetreset_is_present OK")
