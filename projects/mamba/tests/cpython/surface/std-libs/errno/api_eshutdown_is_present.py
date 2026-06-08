# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eshutdown_is_present"
# subject = "errno.ESHUTDOWN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ESHUTDOWN: api_eshutdown_is_present (surface)."""
import errno

assert hasattr(errno, "ESHUTDOWN")
print("api_eshutdown_is_present OK")
