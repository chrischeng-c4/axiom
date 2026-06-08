# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_continuous_is_present"
# subject = "enum.CONTINUOUS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.CONTINUOUS: api_continuous_is_present (surface)."""
import enum

assert hasattr(enum, "CONTINUOUS")
print("api_continuous_is_present OK")
