# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "surface"
# case = "api_unregister_is_present"
# subject = "atexit.unregister"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""atexit.unregister: api_unregister_is_present (surface)."""
import atexit

assert hasattr(atexit, "unregister")
print("api_unregister_is_present OK")
