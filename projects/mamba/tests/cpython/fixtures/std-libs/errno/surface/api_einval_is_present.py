# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_einval_is_present"
# subject = "errno.EINVAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EINVAL: api_einval_is_present (surface)."""
import errno

assert hasattr(errno, "EINVAL")
print("api_einval_is_present OK")
