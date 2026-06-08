# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_espipe_is_present"
# subject = "errno.ESPIPE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ESPIPE: api_espipe_is_present (surface)."""
import errno

assert hasattr(errno, "ESPIPE")
print("api_espipe_is_present OK")
