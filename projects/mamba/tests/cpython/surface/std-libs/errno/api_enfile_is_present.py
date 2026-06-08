# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enfile_is_present"
# subject = "errno.ENFILE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENFILE: api_enfile_is_present (surface)."""
import errno

assert hasattr(errno, "ENFILE")
print("api_enfile_is_present OK")
