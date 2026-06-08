# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_repr_enum_is_present"
# subject = "enum.ReprEnum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.ReprEnum: api_repr_enum_is_present (surface)."""
import enum

assert hasattr(enum, "ReprEnum")
print("api_repr_enum_is_present OK")
