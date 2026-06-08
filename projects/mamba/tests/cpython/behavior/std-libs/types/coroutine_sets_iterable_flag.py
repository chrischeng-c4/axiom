# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_sets_iterable_flag"
# subject = "types.coroutine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.coroutine: coroutine() sets the CO_ITERABLE_COROUTINE code flag without marking the generator a native CO_COROUTINE, visible on the function and a live generator's code object"""
import inspect
import types


def gen():
    yield


types.coroutine(gen)

# coroutine() sets the iterable-coroutine code flag without marking it a
# native coroutine. The flag also shows on a live generator's code object.
assert gen.__code__.co_flags & inspect.CO_ITERABLE_COROUTINE
assert not gen.__code__.co_flags & inspect.CO_COROUTINE
running = gen()
assert running.gi_code.co_flags & inspect.CO_ITERABLE_COROUTINE
assert not running.gi_code.co_flags & inspect.CO_COROUTINE

print("coroutine_sets_iterable_flag OK")
