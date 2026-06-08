# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_int_enum_is_present"
# subject = "enum.IntEnum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.IntEnum: api_int_enum_is_present (surface)."""
import enum

assert hasattr(enum, "IntEnum")
print("api_int_enum_is_present OK")
