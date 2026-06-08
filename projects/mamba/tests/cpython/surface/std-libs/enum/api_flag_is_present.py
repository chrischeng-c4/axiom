# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_flag_is_present"
# subject = "enum.Flag"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.Flag: api_flag_is_present (surface)."""
import enum

assert hasattr(enum, "Flag")
print("api_flag_is_present OK")
