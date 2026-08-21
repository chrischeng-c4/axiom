# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "errors"
# case = "unquote_none_arg_raises"
# subject = "urllib.parse.unquote"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote: unquote_none_arg_raises (errors)."""
from urllib.parse import unquote

_raised = False
try:
    unquote(None)
except TypeError:
    _raised = True
assert _raised, "unquote_none_arg_raises: expected TypeError"
print("unquote_none_arg_raises OK")
