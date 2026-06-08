# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "errors"
# case = "coroutine_wrapped_arg_mismatch_raises"
# subject = "types.coroutine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.coroutine: coroutine() returns a wrapper that forwards the call to the wrapped plain function, so calling it with an extra positional arg the function does not accept raises TypeError"""
import types


def regular() -> int:
    return 1


# coroutine() wraps a plain (non-generator) function; the wrapper forwards the
# call, so passing an argument that regular() does not accept raises TypeError.
_raised = False
try:
    types.coroutine(regular)(1)
except TypeError:
    _raised = True
assert _raised, "coroutine_wrapped_arg_mismatch_raises: expected TypeError"

print("coroutine_wrapped_arg_mismatch_raises OK")
