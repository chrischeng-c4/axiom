# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_get_type_hints_is_present"
# subject = "typing.get_type_hints"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.get_type_hints: api_get_type_hints_is_present (surface)."""
import typing

assert hasattr(typing, "get_type_hints")
print("api_get_type_hints_is_present OK")
