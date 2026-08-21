# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_utf8_default_encoding"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: default (or encoding=None) UTF-8 encoding turns each non-ASCII char into its UTF-8 byte escapes; latin-1 produces single-byte escapes"""
from urllib.parse import quote

assert quote("\xa2\xd8ab\xff") == "%C2%A2%C3%98ab%C3%BF", "utf-8 default"
assert quote("\u6f22\u5b57") == "%E6%BC%A2%E5%AD%97", "utf-8 CJK"
assert quote("\xa2\xd8ab\xff", encoding=None, errors=None) == \
    "%C2%A2%C3%98ab%C3%BF", "None encoding == utf-8"
assert quote("\xa2\xd8ab\xff", encoding="latin-1") == "%A2%D8ab%FF", "latin-1"

print("quote_utf8_default_encoding OK")
