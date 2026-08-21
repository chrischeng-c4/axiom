# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "context_decorator_as_with_block"
# subject = "contextlib.ContextDecorator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ContextDecorator: a ContextDecorator instance works as a plain with-block, running enter/body/exit in order"""
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


with Track("blk"):
    log.append("body")
assert log == ["enter:blk", "body", "exit:blk"], log

print("context_decorator_as_with_block OK")
