# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "mutable_mapping_mixin_methods"
# subject = "collections.abc.MutableMapping"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.MutableMapping: nominal subclasses inherit mutating mapping mixins"""
import collections.abc as abc


class MyMutableMapping(abc.MutableMapping):
    def __init__(self, pairs=()):
        self.data = dict(pairs)

    def __getitem__(self, key):
        return self.data[key]

    def __setitem__(self, key, value):
        self.data[key] = value

    def __delitem__(self, key):
        del self.data[key]

    def __iter__(self):
        return iter(self.data)

    def __len__(self):
        return len(self.data)


mapping = MyMutableMapping([("red", 5)])

assert mapping.setdefault("red", 99) == 5, "setdefault returns existing value"
assert mapping.setdefault("blue", 7) == 7, "setdefault inserts missing default"
assert mapping.data == {"red": 5, "blue": 7}, "setdefault mutates missing keys"

mapping.update({"green": 9})
mapping.update([("orange", 3)])
assert mapping.data == {"red": 5, "blue": 7, "green": 9, "orange": 3}, "update accepts mappings and pair iterables"

assert mapping.pop("green") == 9, "pop removes existing key"
assert mapping.pop("missing", 42) == 42, "pop returns explicit default"
try:
    mapping.pop("missing")
    raise AssertionError("pop without default should reject missing keys")
except KeyError:
    pass

removed_key, removed_value = mapping.popitem()
assert removed_key not in mapping, "popitem removes the returned key"
assert removed_value in (3, 5, 7), "popitem returns a stored value"

mapping.clear()
assert mapping.data == {}, "clear removes all items"

mapping.update(red=5, blue=7)
assert mapping.data == {"red": 5, "blue": 7}, "update accepts keyword items"
assert list(mapping.keys()) == ["red", "blue"], "MutableMapping inherits Mapping keys"
assert list(mapping.items()) == [("red", 5), ("blue", 7)], "MutableMapping inherits Mapping items"
assert "red" in mapping, "MutableMapping inherits Mapping __contains__"

native = {"a": 1}
native.update({"b": 2})
assert native.setdefault("b", 9) == 2, "native dict setdefault behavior remains intact"
assert native.pop("a") == 1, "native dict pop behavior remains intact"

print("mutable_mapping_mixin_methods OK")
