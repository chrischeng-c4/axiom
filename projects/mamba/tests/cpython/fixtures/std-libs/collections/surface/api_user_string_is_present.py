# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "api_user_string_is_present"
# subject = "collections.UserString"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""collections.UserString: api_user_string_is_present (surface)."""
import collections

assert hasattr(collections, "UserString")
print("api_user_string_is_present OK")
