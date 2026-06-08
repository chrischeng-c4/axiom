# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_named_flags_is_present"
# subject = "enum.NAMED_FLAGS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.NAMED_FLAGS: api_named_flags_is_present (surface)."""
import enum

assert hasattr(enum, "NAMED_FLAGS")
print("api_named_flags_is_present OK")
