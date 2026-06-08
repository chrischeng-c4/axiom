# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_type_var_is_present"
# subject = "typing.TypeVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.TypeVar: api_type_var_is_present (surface)."""
import typing

assert hasattr(typing, "TypeVar")
print("api_type_var_is_present OK")
