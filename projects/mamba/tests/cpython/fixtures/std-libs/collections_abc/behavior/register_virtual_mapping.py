# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "register_virtual_mapping"
# subject = "collections.abc.Mapping"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Mapping: Mapping.register() makes an unrelated mapping-shaped class a virtual subclass recognized by isinstance"""
import collections.abc as abc


class CustomMapping:
    def __getitem__(self, key):
        return key

    def __len__(self):
        return 0

    def __iter__(self):
        return iter([])


# Before registration: not a virtual subclass.
assert not isinstance(CustomMapping(), abc.Mapping), "unregistered not a Mapping"
abc.Mapping.register(CustomMapping)
assert isinstance(CustomMapping(), abc.Mapping), "registered Mapping"
assert issubclass(CustomMapping, abc.Mapping), "registered Mapping subclass"
print("register_virtual_mapping OK")
