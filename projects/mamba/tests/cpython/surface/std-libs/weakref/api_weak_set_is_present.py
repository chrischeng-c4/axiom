# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "api_weak_set_is_present"
# subject = "weakref.WeakSet"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""weakref.WeakSet: api_weak_set_is_present (surface)."""
import weakref

assert hasattr(weakref, "WeakSet")
print("api_weak_set_is_present OK")
