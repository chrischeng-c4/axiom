# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_type_is_present"
# subject = "typing.Type"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Type: api_type_is_present (surface)."""
import typing

assert hasattr(typing, "Type")
print("api_type_is_present OK")
