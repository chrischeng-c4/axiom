# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "userdict_delegates_to_data"
# subject = "collections.UserDict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.UserDict: UserDict stores its payload in a public .data dict and delegates mapping ops; a __missing__ override customizes absent-key lookup while .get bypasses it"""
from collections import UserDict

ud = UserDict(a=1)
ud["b"] = 2
assert sorted(ud.items()) == [("a", 1), ("b", 2)], f"items = {sorted(ud.items())!r}"
assert ud.data == {"a": 1, "b": 2}, "payload lives in .data"
assert ud["a"] == 1 and "b" in ud and len(ud) == 2, "delegated mapping ops"

class WithMissing(UserDict):
    def __missing__(self, key):
        return 456

assert WithMissing()[123] == 456, "__missing__ customizes absent-key lookup"
assert WithMissing().get(123) is None, ".get bypasses __missing__"

print("userdict_delegates_to_data OK")
