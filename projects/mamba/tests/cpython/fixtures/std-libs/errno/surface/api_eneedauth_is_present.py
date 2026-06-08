# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eneedauth_is_present"
# subject = "errno.ENEEDAUTH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENEEDAUTH: api_eneedauth_is_present (surface)."""
import errno

assert hasattr(errno, "ENEEDAUTH")
print("api_eneedauth_is_present OK")
