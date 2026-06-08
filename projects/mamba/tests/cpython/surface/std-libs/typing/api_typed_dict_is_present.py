# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_typed_dict_is_present"
# subject = "typing.TypedDict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.TypedDict: api_typed_dict_is_present (surface)."""
import typing

assert hasattr(typing, "TypedDict")
print("api_typed_dict_is_present OK")
