# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enotsup_is_present"
# subject = "errno.ENOTSUP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOTSUP: api_enotsup_is_present (surface)."""
import errno

assert hasattr(errno, "ENOTSUP")
print("api_enotsup_is_present OK")
