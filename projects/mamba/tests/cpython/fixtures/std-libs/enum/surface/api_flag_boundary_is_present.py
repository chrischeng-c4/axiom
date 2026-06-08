# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_flag_boundary_is_present"
# subject = "enum.FlagBoundary"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.FlagBoundary: api_flag_boundary_is_present (surface)."""
import enum

assert hasattr(enum, "FlagBoundary")
print("api_flag_boundary_is_present OK")
