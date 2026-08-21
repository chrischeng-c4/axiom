# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "context_decorator_typo_exit_raises_typeerror"
# subject = "contextlib.ContextDecorator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ContextDecorator: a ContextDecorator subclass that misspells __exit__ raises TypeError at enter time, with a message mentioning the context manager protocol"""
import contextlib


class BadExit(contextlib.ContextDecorator):
    def __enter__(self):
        pass

    def __uxit__(self, *exc):  # typo: not __exit__
        pass


_raised = False
try:
    with BadExit():
        pass
except TypeError as e:
    _raised = True
    assert "context manager" in str(e), str(e)
assert _raised, "expected TypeError for the misspelled __exit__"

print("context_decorator_typo_exit_raises_typeerror OK")
