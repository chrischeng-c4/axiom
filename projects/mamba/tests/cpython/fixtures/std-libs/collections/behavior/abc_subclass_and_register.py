# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "abc_subclass_and_register"
# subject = "collections.abc.Hashable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc.Hashable: direct subclassing of the one-trick-pony ABCs makes issubclass true, and register() makes a structurally-unrelated class a virtual subclass"""
import collections.abc as abc

ponies = (abc.Hashable, abc.Iterable, abc.Iterator, abc.Reversible,
          abc.Sized, abc.Container, abc.Callable)

for base in ponies:
    class Derived(base):
        pass
    assert issubclass(Derived, base), f"subclass of {base.__name__}"
    assert not issubclass(int, Derived), "int is not a subclass of Derived"

for base in ponies:
    class Plain:
        __hash__ = None
    assert not issubclass(Plain, base), f"not yet {base.__name__}"
    base.register(Plain)
    assert issubclass(Plain, base), f"registered as a virtual subclass of {base.__name__}"

print("abc_subclass_and_register OK")
