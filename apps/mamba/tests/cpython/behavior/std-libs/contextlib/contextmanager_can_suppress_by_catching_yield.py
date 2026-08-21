# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "contextmanager_can_suppress_by_catching_yield"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager that wraps `yield` in try/except ValueError swallows a ValueError raised in the body, so it does not propagate"""
import contextlib


@contextlib.contextmanager
def suppress_value_error():
    try:
        yield
    except ValueError:
        pass  # swallow the body's ValueError


reached_after = False
with suppress_value_error():
    raise ValueError("suppressed")  # caught by the manager, does not propagate
reached_after = True
assert reached_after, "execution must continue after a swallowed exception"

print("contextmanager_can_suppress_by_catching_yield OK")
