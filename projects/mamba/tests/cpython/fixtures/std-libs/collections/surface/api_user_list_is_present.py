# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "api_user_list_is_present"
# subject = "collections.UserList"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""collections.UserList: api_user_list_is_present (surface)."""
import collections

assert hasattr(collections, "UserList")
print("api_user_list_is_present OK")
