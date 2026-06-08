# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "bad_escape_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.compile: bad_escape_raises (errors)."""
import re

_raised = False
try:
    re.compile(r'\q')
except re.error:
    _raised = True
assert _raised, "bad_escape_raises: expected re.error"
print("bad_escape_raises OK")
