# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_get_args_is_present"
# subject = "typing.get_args"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.get_args: api_get_args_is_present (surface)."""
import typing

assert hasattr(typing, "get_args")
print("api_get_args_is_present OK")
