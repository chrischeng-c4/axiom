# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_auto_is_present"
# subject = "enum.auto"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.auto: api_auto_is_present (surface)."""
import enum

assert hasattr(enum, "auto")
print("api_auto_is_present OK")
