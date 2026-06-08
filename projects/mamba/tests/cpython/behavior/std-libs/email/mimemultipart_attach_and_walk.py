# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "mimemultipart_attach_and_walk"
# subject = "email.mime.multipart.MIMEMultipart"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.mime.multipart.MIMEMultipart: MIMEMultipart('alternative') with attached plain+html parts: get_content_maintype()=='multipart', payload has the parts, and walk() yields the container plus both text/plain and text/html parts"""
from email.mime.multipart import MIMEMultipart

from email.mime.text import MIMEText

multi = MIMEMultipart("alternative")
multi.attach(MIMEText("Plain text", "plain"))
multi.attach(MIMEText("<b>HTML</b>", "html"))
assert multi.get_content_maintype() == "multipart", "multipart maintype"
payload = multi.get_payload()
assert len(payload) == 2, f"two parts = {len(payload)!r}"
parts = list(multi.walk())
# walk yields the multipart container itself plus the two parts.
assert len(parts) >= 3, f"walk parts = {len(parts)!r}"
ctypes = [p.get_content_type() for p in parts]
assert "text/plain" in ctypes, "plain text part"
assert "text/html" in ctypes, "html part"

print("mimemultipart_attach_and_walk OK")
