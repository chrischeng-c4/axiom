# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_strict_is_present"
# subject = "enum.STRICT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.STRICT: api_strict_is_present (surface)."""
import enum

assert hasattr(enum, "STRICT")
print("api_strict_is_present OK")
