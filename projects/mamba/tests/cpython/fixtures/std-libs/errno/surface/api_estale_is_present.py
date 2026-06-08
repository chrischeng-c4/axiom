# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_estale_is_present"
# subject = "errno.ESTALE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ESTALE: api_estale_is_present (surface)."""
import errno

assert hasattr(errno, "ESTALE")
print("api_estale_is_present OK")
