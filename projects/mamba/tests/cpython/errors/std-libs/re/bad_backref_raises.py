# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "bad_backref_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.compile: bad_backref_raises (errors)."""
import re

_raised = False
try:
    re.compile(r'(\1)')
except re.error:
    _raised = True
assert _raised, "bad_backref_raises: expected re.error"
print("bad_backref_raises OK")
