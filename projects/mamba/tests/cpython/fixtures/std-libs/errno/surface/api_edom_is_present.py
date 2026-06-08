# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_edom_is_present"
# subject = "errno.EDOM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EDOM: api_edom_is_present (surface)."""
import errno

assert hasattr(errno, "EDOM")
print("api_edom_is_present OK")
