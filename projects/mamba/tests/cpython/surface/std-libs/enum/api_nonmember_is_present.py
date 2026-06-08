# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_nonmember_is_present"
# subject = "enum.nonmember"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.nonmember: api_nonmember_is_present (surface)."""
import enum

assert hasattr(enum, "nonmember")
print("api_nonmember_is_present OK")
