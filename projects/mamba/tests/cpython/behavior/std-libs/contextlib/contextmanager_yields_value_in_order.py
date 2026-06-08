# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "contextmanager_yields_value_in_order"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager runs the pre-yield body on enter, yields its value to `as`, then runs the post-yield body on exit, in that order (before/during/after)"""
import contextlib

order: list = []


@contextlib.contextmanager
def cm():
    order.append("before")
    yield "value"
    order.append("after")


with cm() as v:
    assert v == "value", f"yield value = {v!r}"
    order.append("during")
assert order == ["before", "during", "after"], f"order = {order!r}"

print("contextmanager_yields_value_in_order OK")
