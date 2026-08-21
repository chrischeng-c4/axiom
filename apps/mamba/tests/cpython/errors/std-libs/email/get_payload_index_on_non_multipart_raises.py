# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "errors"
# case = "get_payload_index_on_non_multipart_raises"
# subject = "email.message.Message"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: get_payload_index_on_non_multipart_raises (errors)."""
from email.message import Message

_raised = False
try:
    Message().get_payload(1)
except TypeError:
    _raised = True
assert _raised, "get_payload_index_on_non_multipart_raises: expected TypeError"
print("get_payload_index_on_non_multipart_raises OK")
