# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_ebadmacho_is_present"
# subject = "errno.EBADMACHO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EBADMACHO: api_ebadmacho_is_present (surface)."""
import errno

assert hasattr(errno, "EBADMACHO")
print("api_ebadmacho_is_present OK")
