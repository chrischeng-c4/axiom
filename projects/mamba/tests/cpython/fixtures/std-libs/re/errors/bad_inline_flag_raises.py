# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "bad_inline_flag_raises"
# subject = "re.compile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.compile: bad_inline_flag_raises (errors)."""
import re

_raised = False
try:
    re.compile(r'(?z)')
except re.error:
    _raised = True
assert _raised, "bad_inline_flag_raises: expected re.error"
print("bad_inline_flag_raises OK")
