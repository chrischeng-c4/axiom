# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eremote_is_present"
# subject = "errno.EREMOTE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EREMOTE: api_eremote_is_present (surface)."""
import errno

assert hasattr(errno, "EREMOTE")
print("api_eremote_is_present OK")
