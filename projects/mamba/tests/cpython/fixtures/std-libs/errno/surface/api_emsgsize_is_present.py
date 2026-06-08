# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_emsgsize_is_present"
# subject = "errno.EMSGSIZE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EMSGSIZE: api_emsgsize_is_present (surface)."""
import errno

assert hasattr(errno, "EMSGSIZE")
print("api_emsgsize_is_present OK")
