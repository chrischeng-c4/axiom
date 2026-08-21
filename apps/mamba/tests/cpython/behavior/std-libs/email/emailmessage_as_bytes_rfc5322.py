# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "emailmessage_as_bytes_rfc5322"
# subject = "email.message.EmailMessage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.EmailMessage: EmailMessage.as_bytes() returns a bytes serialization carrying the From header and the body bytes"""
from email.message import EmailMessage

msg = EmailMessage()
msg["From"] = "sender@example.com"
msg.set_content("Bytes body")
b = msg.as_bytes()
assert isinstance(b, bytes), f"as_bytes type = {type(b)!r}"
assert b"From:" in b, "From in bytes"
assert b"Bytes body" in b, "body in bytes"

print("emailmessage_as_bytes_rfc5322 OK")
