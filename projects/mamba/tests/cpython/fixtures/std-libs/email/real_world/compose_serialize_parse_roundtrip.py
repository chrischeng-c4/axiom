# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "real_world"
# case = "compose_serialize_parse_roundtrip"
# subject = "email.message.EmailMessage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.EmailMessage: an outbound-mail flow composes a multipart EmailMessage (From/To/Subject + plain body + an attached part), serializes it to bytes, then re-parses it with message_from_bytes and recovers every header, the body, and the attachment payload"""
import email

from email.message import EmailMessage
from email.policy import default as default_policy

# An outbound-mail flow: compose a multipart message with a plain body plus an
# attached part, serialize to bytes, then re-parse and recover everything.
out = EmailMessage()
out["From"] = "alice@example.com"
out["To"] = "bob@example.com"
out["Subject"] = "Report"
out.set_content("Please find the report attached.\n")
out.add_attachment(
    b"col1,col2\n1,2\n",
    maintype="text",
    subtype="csv",
    filename="report.csv",
)

blob = out.as_bytes()
assert isinstance(blob, bytes), "as_bytes returns bytes"

parsed = email.message_from_bytes(blob, policy=default_policy)
assert parsed["From"] == "alice@example.com", f"From = {parsed['From']!r}"
assert parsed["To"] == "bob@example.com", f"To = {parsed['To']!r}"
assert parsed["Subject"] == "Report", f"Subject = {parsed['Subject']!r}"
assert parsed.is_multipart(), "round-tripped message is multipart"

bodies = [p for p in parsed.walk() if p.get_content_type() == "text/plain"]
assert bodies, "plain body part recovered"
assert "Please find the report attached." in bodies[0].get_content(), "body text recovered"

attachments = [p for p in parsed.walk() if p.get_filename() == "report.csv"]
assert attachments, "attachment recovered by filename"
assert attachments[0].get_content_type() == "text/csv", attachments[0].get_content_type()
payload = attachments[0].get_payload(decode=True)
assert payload == b"col1,col2\n1,2\n", repr(payload)

print("compose_serialize_parse_roundtrip OK")
