# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_conform_is_present"
# subject = "enum.CONFORM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.CONFORM: api_conform_is_present (surface)."""
import enum

assert hasattr(enum, "CONFORM")
print("api_conform_is_present OK")
