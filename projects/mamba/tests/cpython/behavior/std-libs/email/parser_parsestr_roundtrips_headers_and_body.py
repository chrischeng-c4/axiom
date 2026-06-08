# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "parser_parsestr_roundtrips_headers_and_body"
# subject = "email.parser.Parser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.parser.Parser: Parser().parsestr round-trips a raw RFC 5322 message: From/Subject headers and the body text are recovered"""
from email.parser import Parser

p = Parser()
original = "From: alice@example.com\r\nTo: bob@example.com\r\nSubject: Parse\r\n\r\nBody text.\r\n"
m = p.parsestr(original)
assert m["From"] == "alice@example.com", f"parsed From = {m['From']!r}"
assert m["Subject"] == "Parse", f"parsed Subject = {m['Subject']!r}"
assert "Body text." in m.get_payload(), f"parsed body = {m.get_payload()!r}"

print("parser_parsestr_roundtrips_headers_and_body OK")
