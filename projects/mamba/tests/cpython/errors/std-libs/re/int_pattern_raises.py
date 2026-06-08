# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "int_pattern_raises"
# subject = "re.match"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.match: int_pattern_raises (errors)."""
import re

_raised = False
try:
    re.match(123, 'abc')
except TypeError:
    _raised = True
assert _raised, "int_pattern_raises: expected TypeError"
print("int_pattern_raises OK")
