# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "custom_iter_is_iterable"
# subject = "collections.abc.Iterable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Iterable: a class defining __iter__ is recognized as Iterable via structural subclass hook, while a plain object is not"""
import collections.abc as abc


class MyIterable:
    def __iter__(self):
        return iter([1, 2, 3])


assert isinstance(MyIterable(), abc.Iterable), "custom __iter__ is Iterable"
assert not isinstance(object(), abc.Iterable), "plain object not Iterable"
print("custom_iter_is_iterable OK")
