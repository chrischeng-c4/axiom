# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "custom_contains_is_container"
# subject = "collections.abc.Container"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Container: a class defining __contains__ is a Container and drives the `in` operator"""
import collections.abc as abc


class MyContainer:
    def __contains__(self, item):
        return item in {1, 2, 3}


assert isinstance(MyContainer(), abc.Container), "custom __contains__ is Container"
assert 1 in MyContainer(), "container contains 1"
assert 9 not in MyContainer(), "container lacks 9"
print("custom_contains_is_container OK")
