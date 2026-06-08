# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_is_typeddict_is_present"
# subject = "typing.is_typeddict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.is_typeddict: api_is_typeddict_is_present (surface)."""
import typing

assert hasattr(typing, "is_typeddict")
print("api_is_typeddict_is_present OK")
