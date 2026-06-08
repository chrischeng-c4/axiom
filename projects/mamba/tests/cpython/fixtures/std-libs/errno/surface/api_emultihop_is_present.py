# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_emultihop_is_present"
# subject = "errno.EMULTIHOP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EMULTIHOP: api_emultihop_is_present (surface)."""
import errno

assert hasattr(errno, "EMULTIHOP")
print("api_emultihop_is_present OK")
