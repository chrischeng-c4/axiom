# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "tracebackexception_limit_trims_stack"
# subject = "traceback.TracebackException"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.TracebackException: from_exception(e, limit=5) trims the captured stack to 5 frames, matching StackSummary.extract(walk_tb(tb), limit=5)"""
import traceback


def recurse(n):
    if n:
        recurse(n - 1)
    else:
        1 / 0


try:
    recurse(10)
except ZeroDivisionError as e:
    limited = traceback.TracebackException.from_exception(e, limit=5)
    expected = traceback.StackSummary.extract(traceback.walk_tb(e.__traceback__), limit=5)
assert limited.stack == expected, "limit trims stack consistently"
assert len(limited.stack) == 5, f"limited stack len = {len(limited.stack)!r}"

print("tracebackexception_limit_trims_stack OK")
