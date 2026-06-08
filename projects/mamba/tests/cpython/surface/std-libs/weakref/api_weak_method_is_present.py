# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "api_weak_method_is_present"
# subject = "weakref.WeakMethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""weakref.WeakMethod: api_weak_method_is_present (surface)."""
import weakref

assert hasattr(weakref, "WeakMethod")
print("api_weak_method_is_present OK")
