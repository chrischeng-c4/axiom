# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_verify_is_present"
# subject = "enum.verify"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.verify: api_verify_is_present (surface)."""
import enum

assert hasattr(enum, "verify")
print("api_verify_is_present OK")
