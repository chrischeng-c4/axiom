# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "surface"
# case = "api_register_is_present"
# subject = "atexit.register"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""atexit.register: api_register_is_present (surface)."""
import atexit

assert hasattr(atexit, "register")
print("api_register_is_present OK")
