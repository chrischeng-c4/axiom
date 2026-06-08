# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_enum_check_is_present"
# subject = "enum.EnumCheck"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.EnumCheck: api_enum_check_is_present (surface)."""
import enum

assert hasattr(enum, "EnumCheck")
print("api_enum_check_is_present OK")
