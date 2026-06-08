# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "custom_len_is_sized"
# subject = "collections.abc.Sized"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Sized: a class defining __len__ is recognized as Sized via the structural subclass hook"""
import collections.abc as abc


class MySized:
    def __len__(self):
        return 5


assert isinstance(MySized(), abc.Sized), "custom __len__ is Sized"
assert len(MySized()) == 5, "len delegates to __len__"
print("custom_len_is_sized OK")
