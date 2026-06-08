# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "parse_qs_honours_encoding"
# subject = "urllib.parse.parse_qs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.parse_qs: parse_qs/parse_qsl decode percent-escapes under the requested encoding= ('%C3%A9' as utf-8 and '%E9' as latin-1 both yield 'é'), and errors='ignore' drops undecodable bytes; bytes input yields bytes pairs"""
from urllib.parse import parse_qs, parse_qsl

assert parse_qs("key=%C3%A9", encoding="utf-8") == {"key": ["é"]}
assert parse_qs("key=%E9", encoding="latin-1") == {"key": ["é"]}
assert parse_qs("key=%E9-", encoding="ascii", errors="ignore") == {"key": ["-"]}

assert parse_qsl("key=%C3%A9", encoding="utf-8") == [("key", "é")]
assert parse_qsl(b"a=b") == [(b"a", b"b")]
assert parse_qsl(bytearray(b"a=b")) == [(b"a", b"b")]

print("parse_qs_honours_encoding OK")
