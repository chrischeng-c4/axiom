# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "api_weak_key_dictionary_is_present"
# subject = "weakref.WeakKeyDictionary"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""weakref.WeakKeyDictionary: api_weak_key_dictionary_is_present (surface)."""
import weakref

assert hasattr(weakref, "WeakKeyDictionary")
print("api_weak_key_dictionary_is_present OK")
