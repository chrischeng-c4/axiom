# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_edquot_is_present"
# subject = "errno.EDQUOT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EDQUOT: api_edquot_is_present (surface)."""
import errno

assert hasattr(errno, "EDQUOT")
print("api_edquot_is_present OK")
