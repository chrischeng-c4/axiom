# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enotty_is_present"
# subject = "errno.ENOTTY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOTTY: api_enotty_is_present (surface)."""
import errno

assert hasattr(errno, "ENOTTY")
print("api_enotty_is_present OK")
