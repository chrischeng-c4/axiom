# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "register_virtual_mutablesequence_no_mixins"
# subject = "collections.abc.MutableSequence.register"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.MutableSequence.register: virtual registration does not install mixin methods"""
import collections.abc as abc


class VirtualSeq:
    def __len__(self):
        return 0

    def __getitem__(self, index):
        raise IndexError(index)

    def __setitem__(self, index, value):
        raise IndexError(index)

    def __delitem__(self, index):
        raise IndexError(index)

    def insert(self, index, value):
        raise AssertionError("virtual sequence insert should not be called")


assert not hasattr(VirtualSeq, "append"), "unregistered class has no append mixin"
assert not hasattr(VirtualSeq(), "append"), "unregistered instance has no append mixin"

abc.MutableSequence.register(VirtualSeq)

assert issubclass(VirtualSeq, abc.MutableSequence), "registered class is a virtual subclass"
assert isinstance(VirtualSeq(), abc.MutableSequence), "registered instance is a virtual instance"
assert not hasattr(VirtualSeq, "append"), "virtual registration does not install class append"
assert not hasattr(VirtualSeq(), "append"), "virtual registration does not install instance append"

try:
    VirtualSeq().append("x")
    raise AssertionError("virtual registration must not provide append")
except AttributeError:
    pass


class NominalSeq(abc.MutableSequence):
    def __init__(self):
        self.items = []

    def __len__(self):
        return len(self.items)

    def __getitem__(self, index):
        return self.items[index]

    def __setitem__(self, index, value):
        self.items[index] = value

    def __delitem__(self, index):
        del self.items[index]

    def insert(self, index, value):
        self.items.insert(index, value)


nominal = NominalSeq()
assert hasattr(NominalSeq, "append"), "nominal subclass inherits append mixin"
assert hasattr(nominal, "append"), "nominal instance exposes append mixin"
nominal.append("x")
assert nominal.items == ["x"], "nominal append mixin remains available"

print("register_virtual_mutablesequence_no_mixins OK")
