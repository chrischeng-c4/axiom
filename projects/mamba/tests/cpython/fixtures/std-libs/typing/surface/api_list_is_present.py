# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_list_is_present"
# subject = "typing.List"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.List: api_list_is_present (surface)."""
import typing

assert hasattr(typing, "List")
print("api_list_is_present OK")
