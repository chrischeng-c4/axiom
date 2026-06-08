# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "userlist_delegates_to_data"
# subject = "collections.UserList"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.UserList: UserList wraps a list in .data and forwards append/extend, iteration, indexing/len, and concatenation"""
from collections import UserList

ul = UserList([1, 2, 3])
ul.append(4)
ul.extend([5])
assert ul.data == [1, 2, 3, 4, 5], "payload lives in .data"
assert list(ul) == [1, 2, 3, 4, 5], "iteration"
assert ul[0] == 1 and ul[-1] == 5 and len(ul) == 5, "indexing/len"
assert (ul + [6])[-1] == 6, "concatenation"

print("userlist_delegates_to_data OK")
