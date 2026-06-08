# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_get_origin_is_present"
# subject = "typing.get_origin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.get_origin: api_get_origin_is_present (surface)."""
import typing

assert hasattr(typing, "get_origin")
print("api_get_origin_is_present OK")
