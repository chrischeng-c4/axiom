# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "api_getweakrefcount_is_present"
# subject = "weakref.getweakrefcount"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""weakref.getweakrefcount: api_getweakrefcount_is_present (surface)."""
import weakref

assert hasattr(weakref, "getweakrefcount")
print("api_getweakrefcount_is_present OK")
