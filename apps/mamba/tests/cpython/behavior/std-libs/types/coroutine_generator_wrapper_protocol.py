# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_generator_wrapper_protocol"
# subject = "types.coroutine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.coroutine: wrapping a function returning a non-coroutine generator yields a generator-wrapper whose repr==str and which exposes the coroutine/generator protocol (__await__/__iter__/send/close/throw)"""
import types


# Wrapping a plain function that returns a NON-coroutine generator yields a
# generator-wrapper: its repr and str agree and it exposes the coroutine /
# generator protocol (a generator that is already coroutine-flagged would be
# returned bare, so use a fresh, unwrapped generator here).
def plain_gen():
    yield


@types.coroutine
def returns_plain_gen():
    return plain_gen()


wrapper = returns_plain_gen()
assert repr(wrapper) == str(wrapper)
expected = {"__await__", "__iter__", "send", "close", "throw"}
assert expected.issubset(set(dir(wrapper)))

print("coroutine_generator_wrapper_protocol OK")
