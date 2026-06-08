# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_ebadarch_is_present"
# subject = "errno.EBADARCH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EBADARCH: api_ebadarch_is_present (surface)."""
import errno

assert hasattr(errno, "EBADARCH")
print("api_ebadarch_is_present OK")
