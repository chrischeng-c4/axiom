# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "exitstack_callbacks_run_lifo"
# subject = "contextlib.ExitStack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ExitStack: ExitStack.callback returns the registered function unchanged and replays stored args/kwargs at unwind time, in LIFO order"""
import contextlib

calls: list = []


def record(*args, **kwds):
    calls.append((args, kwds))


with contextlib.ExitStack() as stack:
    returned = stack.callback(record, 1, key="v")
    stack.callback(record, 2)
    assert returned is record, "callback returns the function unchanged"
# LIFO: the second-registered callback fires first.
assert calls == [((2,), {}), ((1,), {"key": "v"})], calls

print("exitstack_callbacks_run_lifo OK")
