# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "errors"
# case = "message_from_bytes_str_raises"
# subject = "email.message_from_bytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message_from_bytes: message_from_bytes_str_raises (errors)."""
import email

_raised = False
try:
    email.message_from_bytes("not bytes")
except AttributeError:
    _raised = True
assert _raised, "message_from_bytes_str_raises: expected AttributeError"
print("message_from_bytes_str_raises OK")
