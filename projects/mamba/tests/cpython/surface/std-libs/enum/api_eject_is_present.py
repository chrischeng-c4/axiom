# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_eject_is_present"
# subject = "enum.EJECT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.EJECT: api_eject_is_present (surface)."""
import enum

assert hasattr(enum, "EJECT")
print("api_eject_is_present OK")
