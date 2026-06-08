# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "api_update_abstractmethods_is_present"
# subject = "abc.update_abstractmethods"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""abc.update_abstractmethods: api_update_abstractmethods_is_present (surface)."""
import abc

assert hasattr(abc, "update_abstractmethods")
print("api_update_abstractmethods_is_present OK")
