# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eauth_is_present"
# subject = "errno.EAUTH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EAUTH: api_eauth_is_present (surface)."""
import errno

assert hasattr(errno, "EAUTH")
print("api_eauth_is_present OK")
