# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "emailmessage_duplicate_header_kept"
# subject = "email.message.EmailMessage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.EmailMessage: setting the same header name twice keeps both occurrences; items() reports the duplicate name twice"""
from email.message import EmailMessage

msg = EmailMessage()
msg["X-Header-One"] = "val1"
msg["X-Header-Two"] = "val2"
msg["X-Header-One"] = "val3"  # duplicate name
hdr_names = [k for k, v in msg.items()]
assert hdr_names.count("X-Header-One") == 2, f"duplicate headers: {hdr_names!r}"

print("emailmessage_duplicate_header_kept OK")
