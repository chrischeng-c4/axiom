# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_emfile_is_present"
# subject = "errno.EMFILE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EMFILE: api_emfile_is_present (surface)."""
import errno

assert hasattr(errno, "EMFILE")
print("api_emfile_is_present OK")
