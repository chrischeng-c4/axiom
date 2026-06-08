# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_prepare_class_is_present"
# subject = "types.prepare_class"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.prepare_class: api_prepare_class_is_present (surface)."""
import types

assert hasattr(types, "prepare_class")
print("api_prepare_class_is_present OK")
