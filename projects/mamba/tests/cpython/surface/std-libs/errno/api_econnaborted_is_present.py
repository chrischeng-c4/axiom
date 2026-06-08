# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_econnaborted_is_present"
# subject = "errno.ECONNABORTED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ECONNABORTED: api_econnaborted_is_present (surface)."""
import errno

assert hasattr(errno, "ECONNABORTED")
print("api_econnaborted_is_present OK")
