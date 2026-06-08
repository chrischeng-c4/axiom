# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eisdir_is_present"
# subject = "errno.EISDIR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EISDIR: api_eisdir_is_present (surface)."""
import errno

assert hasattr(errno, "EISDIR")
print("api_eisdir_is_present OK")
