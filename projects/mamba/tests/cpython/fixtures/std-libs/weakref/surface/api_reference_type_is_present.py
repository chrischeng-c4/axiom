# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "api_reference_type_is_present"
# subject = "weakref.ReferenceType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""weakref.ReferenceType: api_reference_type_is_present (surface)."""
import weakref

assert hasattr(weakref, "ReferenceType")
print("api_reference_type_is_present OK")
