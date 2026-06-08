# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_str_enum_is_present"
# subject = "enum.StrEnum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.StrEnum: api_str_enum_is_present (surface)."""
import enum

assert hasattr(enum, "StrEnum")
print("api_str_enum_is_present OK")
