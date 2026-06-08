# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "errors"
# case = "quote_int_raises"
# subject = "shlex.quote"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.quote: quote_int_raises (errors)."""
import shlex

_raised = False
try:
    shlex.quote(42)
except TypeError:
    _raised = True
assert _raised, "quote_int_raises: expected TypeError"
print("quote_int_raises OK")
