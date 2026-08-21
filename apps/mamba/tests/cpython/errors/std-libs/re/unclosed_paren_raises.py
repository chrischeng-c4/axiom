# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "unclosed_paren_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.compile: unclosed_paren_raises (errors)."""
import re

_raised = False
try:
    re.compile(r'(unclosed')
except re.error:
    _raised = True
assert _raised, "unclosed_paren_raises: expected re.error"
print("unclosed_paren_raises OK")
