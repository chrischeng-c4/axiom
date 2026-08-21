# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "errors"
# case = "header_bad_charset_raises"
# subject = "email.header.Header"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.header.Header: header_bad_charset_raises (errors)."""
from email.header import Header

_raised = False
try:
    Header("hi", charset="no_such_charset")
except LookupError:
    _raised = True
assert _raised, "header_bad_charset_raises: expected LookupError"
print("header_bad_charset_raises OK")
