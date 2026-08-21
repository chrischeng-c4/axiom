# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "exitstack_propagation_order_keeps_earliest"
# subject = "contextlib.ExitStack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ExitStack: with callbacks running LIFO, a suppressor swallows the later-raised exception and the earlier raised exception is the one that propagates out"""
import contextlib


def raise_exc(exc):
    raise exc


def suppress_all(*exc_details):
    return True


# Callbacks unwind LIFO: the last-registered IndexError raiser runs first, the
# suppressor swallows it, and the first-registered KeyError raiser runs last —
# so KeyError is what propagates.
caught = None
try:
    with contextlib.ExitStack() as stack:
        stack.callback(raise_exc, KeyError("earliest"))
        stack.push(suppress_all)
        stack.callback(raise_exc, IndexError("latest"))
except Exception as exc:
    caught = exc
assert isinstance(caught, KeyError), type(caught).__name__

print("exitstack_propagation_order_keeps_earliest OK")
