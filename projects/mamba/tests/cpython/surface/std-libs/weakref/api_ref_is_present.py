# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "api_ref_is_present"
# subject = "weakref.ref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""weakref.ref: api_ref_is_present (surface)."""
import weakref

assert hasattr(weakref, "ref")
print("api_ref_is_present OK")
