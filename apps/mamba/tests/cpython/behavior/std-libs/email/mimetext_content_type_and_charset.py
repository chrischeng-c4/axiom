# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "mimetext_content_type_and_charset"
# subject = "email.mime.text.MIMEText"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.mime.text.MIMEText: MIMEText(body, subtype, charset) reports get_content_type() == 'text/<subtype>' and get_content_charset() reflecting the declared charset (utf-8), with None/str when omitted"""
from email.mime.text import MIMEText

mime = MIMEText("Plain text content", "plain", "utf-8")
assert isinstance(mime, MIMEText), "MIMEText type"
assert mime.get_content_type() == "text/plain", f"type = {mime.get_content_type()!r}"
assert mime.get_content_charset() == "utf-8", f"charset = {mime.get_content_charset()!r}"

# Without an explicit charset the charset is either None or a str (us-ascii).
plain = MIMEText("hello", "plain")
cs = plain.get_content_charset()
assert cs is None or isinstance(cs, str), f"charset type = {type(cs)!r}"

print("mimetext_content_type_and_charset OK")
