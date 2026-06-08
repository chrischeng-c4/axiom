# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_attribute_error_is_present"
# subject = "builtins.AttributeError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.AttributeError: api_attribute_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "AttributeError")
print("api_attribute_error_is_present OK")
