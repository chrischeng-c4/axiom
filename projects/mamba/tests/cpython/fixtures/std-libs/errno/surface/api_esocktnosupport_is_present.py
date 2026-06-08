# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_esocktnosupport_is_present"
# subject = "errno.ESOCKTNOSUPPORT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ESOCKTNOSUPPORT: api_esocktnosupport_is_present (surface)."""
import errno

assert hasattr(errno, "ESOCKTNOSUPPORT")
print("api_esocktnosupport_is_present OK")
