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
"""collections.abc.Mapping: register() marks virtual subclasses without installing mapping mixins"""
import collections.abc as abc


class CustomMapping:
    def __getitem__(self, key):
        return key

    def __len__(self):
        return 0

    def __iter__(self):
        return iter([])


# Before registration: not a virtual subclass and no Mapping mixins are installed.
assert not isinstance(CustomMapping(), abc.Mapping), "unregistered not a Mapping"
assert not hasattr(CustomMapping, "get"), "unregistered class has no get mixin"
assert not hasattr(CustomMapping(), "items"), "unregistered instance has no items mixin"

abc.Mapping.register(CustomMapping)

assert isinstance(CustomMapping(), abc.Mapping), "registered Mapping"
assert issubclass(CustomMapping, abc.Mapping), "registered Mapping subclass"
assert not hasattr(CustomMapping, "get"), "virtual registration does not install class get"
assert not hasattr(CustomMapping(), "items"), "virtual registration does not install instance items"

try:
    CustomMapping().get("x")
    raise AssertionError("virtual registration must not provide get")
except AttributeError:
    pass

print("register_virtual_mapping OK")
