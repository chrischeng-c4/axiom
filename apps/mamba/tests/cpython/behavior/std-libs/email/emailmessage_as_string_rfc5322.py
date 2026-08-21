# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "emailmessage_as_string_rfc5322"
# subject = "email.message.EmailMessage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.EmailMessage: EmailMessage with From/To/Subject and set_content serializes via as_string() to text carrying the headers and the body"""
from email.message import EmailMessage

msg = EmailMessage()
msg["From"] = "alice@example.com"
msg["To"] = "bob@example.com"
msg["Subject"] = "Hello"
msg.set_content("Test body here.")
s = msg.as_string()
assert "From:" in s, f"From header present: {s[:80]!r}"
assert "Subject: Hello" in s, f"Subject present: {s[:200]!r}"
assert "Test body here." in s, "body present"

print("emailmessage_as_string_rfc5322 OK")
