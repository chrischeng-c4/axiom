# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_e2_big_is_present"
# subject = "errno.E2BIG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.E2BIG: api_e2_big_is_present (surface)."""
import errno

assert hasattr(errno, "E2BIG")
print("api_e2_big_is_present OK")
