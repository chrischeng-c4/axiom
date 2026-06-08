# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "missing_group_index_raises"
# subject = "re.Match.group"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Match.group: missing_group_index_raises (errors)."""
import re

_raised = False
try:
    re.match(r'(a)(b)', 'ab').group(5)
except IndexError:
    _raised = True
assert _raised, "missing_group_index_raises: expected IndexError"
print("missing_group_index_raises OK")
