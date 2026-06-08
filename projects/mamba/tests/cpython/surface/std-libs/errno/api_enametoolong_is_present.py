# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enametoolong_is_present"
# subject = "errno.ENAMETOOLONG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENAMETOOLONG: api_enametoolong_is_present (surface)."""
import errno

assert hasattr(errno, "ENAMETOOLONG")
print("api_enametoolong_is_present OK")
