# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_header_membership_case_insensitive"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: header membership (`in`) is case-insensitive in both directions; an absent header reports not-in"""
from email.message import Message

m = Message()
m["From"] = "Me"
m["to"] = "You"
for probe in ("from", "From", "FROM", "to", "To", "TO"):
    assert probe in m, f"{probe!r} should be in message"
assert "missing" not in m, "absent header reports not-in"

print("message_header_membership_case_insensitive OK")
