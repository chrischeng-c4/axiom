# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_union_is_present"
# subject = "typing.Union"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Union: api_union_is_present (surface)."""
import typing

assert hasattr(typing, "Union")
print("api_union_is_present OK")
