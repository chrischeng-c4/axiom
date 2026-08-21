# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "errors"
# case = "code_replace_nlocals_high_raises"
# subject = "types.CodeType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: code_replace_nlocals_high_raises (errors)."""
import types

_raised = False
try:
    (lambda a, b: a + b).__code__.replace(co_nlocals=3)
except ValueError:
    _raised = True
assert _raised, "code_replace_nlocals_high_raises: expected ValueError"
print("code_replace_nlocals_high_raises OK")
