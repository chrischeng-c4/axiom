# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "multiple_repeat_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.compile: multiple_repeat_raises (errors)."""
import re

_raised = False
try:
    re.compile(r'a**')
except re.error:
    _raised = True
assert _raised, "multiple_repeat_raises: expected re.error"
print("multiple_repeat_raises OK")
