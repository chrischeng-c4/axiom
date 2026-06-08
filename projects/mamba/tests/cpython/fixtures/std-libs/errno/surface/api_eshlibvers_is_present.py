# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eshlibvers_is_present"
# subject = "errno.ESHLIBVERS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ESHLIBVERS: api_eshlibvers_is_present (surface)."""
import errno

assert hasattr(errno, "ESHLIBVERS")
print("api_eshlibvers_is_present OK")
