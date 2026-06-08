# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eownerdead_is_present"
# subject = "errno.EOWNERDEAD"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EOWNERDEAD: api_eownerdead_is_present (surface)."""
import errno

assert hasattr(errno, "EOWNERDEAD")
print("api_eownerdead_is_present OK")
