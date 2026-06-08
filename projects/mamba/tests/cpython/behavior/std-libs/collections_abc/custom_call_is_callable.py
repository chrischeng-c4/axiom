# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "custom_call_is_callable"
# subject = "collections.abc.Callable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Callable: a class defining __call__ is recognized as Callable and is invokable"""
import collections.abc as abc


class MyCallable:
    def __call__(self, x):
        return x * 2


assert isinstance(MyCallable(), abc.Callable), "custom __call__ is Callable"
assert MyCallable()(5) == 10, "callable result"
print("custom_call_is_callable OK")
