# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "suppress_is_reusable_and_reentrant"
# subject = "contextlib.suppress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.suppress: a single suppress() instance can be entered repeatedly (reusable) and nested within itself (reentrant); the outer block resumes after an inner block swallows an exception"""
import contextlib

ignore = contextlib.suppress(Exception)

# REUSABLE: the same instance entered more than once.
with ignore:
    pass
with ignore:
    len(5)  # TypeError, suppressed

# REENTRANT: nesting the same instance; the outer block resumes after the
# inner one swallows an exception.
outer_continued = False
with ignore:
    with ignore:
        len(5)  # suppressed by inner
    outer_continued = True
    1 / 0  # suppressed by outer
assert outer_continued, "outer block must resume after inner suppress"

print("suppress_is_reusable_and_reentrant OK")
