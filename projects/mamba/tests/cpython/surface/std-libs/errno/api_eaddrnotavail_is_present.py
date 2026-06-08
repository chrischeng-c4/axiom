# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eaddrnotavail_is_present"
# subject = "errno.EADDRNOTAVAIL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EADDRNOTAVAIL: api_eaddrnotavail_is_present (surface)."""
import errno

assert hasattr(errno, "EADDRNOTAVAIL")
print("api_eaddrnotavail_is_present OK")
