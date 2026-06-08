# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_unique_is_present_2"
# subject = "enum.unique"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.unique: api_unique_is_present_2 (surface)."""
import enum

assert hasattr(enum, "unique")
print("api_unique_is_present_2 OK")
