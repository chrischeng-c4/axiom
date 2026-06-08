# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enoent_is_present"
# subject = "errno.ENOENT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOENT: api_enoent_is_present (surface)."""
import errno

assert hasattr(errno, "ENOENT")
print("api_enoent_is_present OK")
