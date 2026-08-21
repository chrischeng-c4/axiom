# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_passes_awaitable_through"
# subject = "types.coroutine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.coroutine: wrapping a function that returns a duck-typed awaitable passes the awaitable through and __await__ stays consistent"""
import types


class CoroLike:
    def send(self):
        pass

    def throw(self):
        pass

    def close(self):
        pass

    def __await__(self):
        return self


duck = CoroLike()


@types.coroutine
def returns_duck():
    return duck


assert returns_duck() is duck
assert returns_duck().__await__() is duck

print("coroutine_passes_awaitable_through OK")
