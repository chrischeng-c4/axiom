# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_ecanceled_is_present"
# subject = "errno.ECANCELED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ECANCELED: api_ecanceled_is_present (surface)."""
import errno

assert hasattr(errno, "ECANCELED")
print("api_ecanceled_is_present OK")
