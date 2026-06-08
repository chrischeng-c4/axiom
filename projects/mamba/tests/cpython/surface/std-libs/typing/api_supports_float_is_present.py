# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_supports_float_is_present"
# subject = "typing.SupportsFloat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.SupportsFloat: api_supports_float_is_present (surface)."""
import typing

assert hasattr(typing, "SupportsFloat")
print("api_supports_float_is_present OK")
