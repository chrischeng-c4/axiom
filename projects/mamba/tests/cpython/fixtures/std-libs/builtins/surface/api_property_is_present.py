# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_property_is_present"
# subject = "builtins.property"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.property: api_property_is_present (surface)."""
import builtins

assert hasattr(builtins, "property")
print("api_property_is_present OK")
