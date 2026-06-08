# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_keep_is_present"
# subject = "enum.KEEP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.KEEP: api_keep_is_present (surface)."""
import enum

assert hasattr(enum, "KEEP")
print("api_keep_is_present OK")
