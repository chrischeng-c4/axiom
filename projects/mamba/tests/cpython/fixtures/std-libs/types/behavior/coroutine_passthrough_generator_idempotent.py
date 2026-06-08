# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_passthrough_generator_idempotent"
# subject = "types.coroutine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.coroutine: a generator function passed through coroutine() is returned unchanged, and re-wrapping is idempotent"""
import types


def gen():
    yield


assert types.coroutine(gen) is gen
assert types.coroutine(types.coroutine(gen)) is gen

print("coroutine_passthrough_generator_idempotent OK")
