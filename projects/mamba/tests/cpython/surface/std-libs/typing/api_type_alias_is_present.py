# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_type_alias_is_present"
# subject = "typing.TypeAlias"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.TypeAlias: api_type_alias_is_present (surface)."""
import typing

assert hasattr(typing, "TypeAlias")
print("api_type_alias_is_present OK")
