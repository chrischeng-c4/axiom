# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_attribute_is_present"
# subject = "inspect.Attribute"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.Attribute: api_attribute_is_present (surface)."""
import inspect

assert hasattr(inspect, "Attribute")
print("api_attribute_is_present OK")
