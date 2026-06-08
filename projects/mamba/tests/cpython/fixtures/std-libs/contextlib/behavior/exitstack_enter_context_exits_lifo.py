# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "exitstack_enter_context_exits_lifo"
# subject = "contextlib.ExitStack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ExitStack: ExitStack.enter_context enters each context manager and exits them in LIFO order when the stack closes"""
import contextlib

exits: list = []


@contextlib.contextmanager
def track_exit(name: str):
    try:
        yield
    finally:
        exits.append(name)


with contextlib.ExitStack() as stack:
    stack.enter_context(track_exit("a"))
    stack.enter_context(track_exit("b"))
    stack.enter_context(track_exit("c"))
# LIFO: last entered exits first.
assert exits == ["c", "b", "a"], f"ExitStack LIFO = {exits!r}"

print("exitstack_enter_context_exits_lifo OK")
