# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_ehostdown_is_present"
# subject = "errno.EHOSTDOWN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EHOSTDOWN: api_ehostdown_is_present (surface)."""
import errno

assert hasattr(errno, "EHOSTDOWN")
print("api_ehostdown_is_present OK")
