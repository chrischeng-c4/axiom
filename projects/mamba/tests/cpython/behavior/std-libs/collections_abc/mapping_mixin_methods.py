# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "mapping_mixin_methods"
# subject = "collections.abc.Mapping"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Mapping: nominal subclasses inherit read-only mapping mixins"""
import collections.abc as abc


class MyMapping(abc.Mapping):
    def __init__(self, pairs):
        self.data = dict(pairs)

    def __getitem__(self, key):
        return self.data[key]

    def __iter__(self):
        return iter(self.data)

    def __len__(self):
        return len(self.data)


mapping = MyMapping([("red", 5), ("blue", 7)])

assert "red" in mapping, "__contains__ finds present keys"
assert "green" not in mapping, "__contains__ rejects missing keys"
assert mapping.get("red") == 5, "get returns existing value"
assert mapping.get("green") is None, "get defaults to None"
assert mapping.get("green", 11) == 11, "get returns explicit default"
assert list(mapping.keys()) == ["red", "blue"], "keys view iterates keys"
assert list(mapping.values()) == [5, 7], "values view iterates values"
assert list(mapping.items()) == [("red", 5), ("blue", 7)], "items view iterates pairs"
assert dict(mapping.items()) == {"red": 5, "blue": 7}, "items can materialize a dict"
assert mapping == {"red": 5, "blue": 7}, "mapping equality compares key/value pairs"
assert mapping != {"red": 5}, "mapping inequality detects value differences"

try:
    mapping["green"]
    raise AssertionError("missing mapping key should raise KeyError")
except KeyError:
    pass

print("mapping_mixin_methods OK")
