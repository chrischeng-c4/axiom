# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_enum_type_is_present"
# subject = "enum.EnumType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.EnumType: api_enum_type_is_present (surface)."""
import enum

assert hasattr(enum, "EnumType")
print("api_enum_type_is_present OK")
