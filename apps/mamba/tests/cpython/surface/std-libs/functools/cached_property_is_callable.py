# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "cached_property_is_callable"
# subject = "functools.cached_property"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""functools.cached_property: cached_property_is_callable (surface)."""
import functools

assert callable(functools.cached_property)
print("cached_property_is_callable OK")
