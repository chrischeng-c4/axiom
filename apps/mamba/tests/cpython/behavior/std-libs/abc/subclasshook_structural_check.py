# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "subclasshook_structural_check"
# subject = "abc.ABCMeta"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABCMeta: a custom __subclasshook__ drives structural issubclass checks (has __len__ -> Sized) returning NotImplemented to fall through"""
import abc


class Sized(abc.ABC):
    @classmethod
    def __subclasshook__(cls, C):
        if cls is Sized:
            return hasattr(C, "__len__")
        return NotImplemented


# Anything with __len__ is structurally a Sized.
assert issubclass(list, Sized), "list has __len__ -> Sized"
assert issubclass(str, Sized), "str has __len__ -> Sized"
assert not issubclass(int, Sized), "int has no __len__ -> not Sized"
assert isinstance([1, 2], Sized), "list instance is Sized"
assert not isinstance(7, Sized), "int instance is not Sized"

print("subclasshook_structural_check OK")
