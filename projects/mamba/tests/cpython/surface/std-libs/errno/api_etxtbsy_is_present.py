# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_etxtbsy_is_present"
# subject = "errno.ETXTBSY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ETXTBSY: api_etxtbsy_is_present (surface)."""
import errno

assert hasattr(errno, "ETXTBSY")
print("api_etxtbsy_is_present OK")
