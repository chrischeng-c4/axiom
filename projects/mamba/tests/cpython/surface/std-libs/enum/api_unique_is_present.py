# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_unique_is_present"
# subject = "enum.UNIQUE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.UNIQUE: api_unique_is_present (surface)."""
import enum

assert hasattr(enum, "UNIQUE")
print("api_unique_is_present OK")
