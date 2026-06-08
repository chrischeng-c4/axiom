# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eproclim_is_present"
# subject = "errno.EPROCLIM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EPROCLIM: api_eproclim_is_present (surface)."""
import errno

assert hasattr(errno, "EPROCLIM")
print("api_eproclim_is_present OK")
