# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "bytesparser_parsebytes_recovers_headers"
# subject = "email.parser.BytesParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.parser.BytesParser: BytesParser().parsebytes parses a raw bytes message and recovers the From and Subject headers"""
from email.parser import BytesParser

bp = BytesParser()
raw = b"From: test@example.com\r\nSubject: Bytes\r\n\r\nByte body.\r\n"
m = bp.parsebytes(raw)
assert m["From"] == "test@example.com", f"BytesParser From = {m['From']!r}"
assert m["Subject"] == "Bytes", f"BytesParser Subject = {m['Subject']!r}"

print("bytesparser_parsebytes_recovers_headers OK")
