# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eoverflow_is_present"
# subject = "errno.EOVERFLOW"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EOVERFLOW: api_eoverflow_is_present (surface)."""
import errno

assert hasattr(errno, "EOVERFLOW")
print("api_eoverflow_is_present OK")
