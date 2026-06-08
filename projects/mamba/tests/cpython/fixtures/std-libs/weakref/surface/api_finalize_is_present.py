# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "api_finalize_is_present"
# subject = "weakref.finalize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""weakref.finalize: api_finalize_is_present (surface)."""
import weakref

assert hasattr(weakref, "finalize")
print("api_finalize_is_present OK")
