# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "context_decorator_as_function_decorator"
# subject = "contextlib.ContextDecorator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ContextDecorator: the same ContextDecorator instance applied as a function decorator wraps every call in enter/exit, re-entering the manager per call"""
import contextlib

log: list = []


class Track(contextlib.ContextDecorator):
    def __init__(self, name):
        self.name = name

    def __enter__(self):
        log.append(f"enter:{self.name}")
        return self

    def __exit__(self, *exc):
        log.append(f"exit:{self.name}")
        return False


@Track("fn")
def do_work(x):
    log.append(f"work:{x}")
    return x * 2


assert do_work(3) == 6
assert do_work(4) == 8
# Each call re-enters and re-exits the manager.
assert log == [
    "enter:fn", "work:3", "exit:fn",
    "enter:fn", "work:4", "exit:fn",
], log

print("context_decorator_as_function_decorator OK")
