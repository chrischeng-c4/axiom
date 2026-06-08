# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "nothing_to_repeat_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.compile: nothing_to_repeat_raises (errors)."""
import re

_raised = False
try:
    re.compile(r'*')
except re.error:
    _raised = True
assert _raised, "nothing_to_repeat_raises: expected re.error"
print("nothing_to_repeat_raises OK")
