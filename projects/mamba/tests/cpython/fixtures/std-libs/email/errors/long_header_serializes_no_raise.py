# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "errors"
# case = "long_header_serializes_no_raise"
# subject = "email.message.EmailMessage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.EmailMessage: an oversized 10000-char header value is accepted by the default policy; as_string() folds and serializes it without raising HeaderParseError"""
from email.message import EmailMessage

# The default policy accepts a very long header value and folds it on
# serialization; no HeaderParseError is raised.
m = EmailMessage()
m["From"] = "a" * 10000
s = m.as_string()
assert len(s) > 0, "serialized output should be non-empty"
assert "From:" in s, "From header should survive serialization"

print("long_header_serializes_no_raise OK")
