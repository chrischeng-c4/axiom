# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "userlist_is_callable"
# subject = "collections.UserList"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.UserList: userlist_is_callable (surface)."""
import collections

assert callable(collections.UserList)
print("userlist_is_callable OK")
