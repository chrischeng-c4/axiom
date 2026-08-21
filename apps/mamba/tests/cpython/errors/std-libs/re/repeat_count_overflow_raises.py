# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "repeat_count_overflow_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.compile: repeat_count_overflow_raises (errors)."""
import re

_raised = False
try:
    re.compile('.{%d}' % (2 ** 100))
except OverflowError:
    _raised = True
assert _raised, "repeat_count_overflow_raises: expected OverflowError"
print("repeat_count_overflow_raises OK")
