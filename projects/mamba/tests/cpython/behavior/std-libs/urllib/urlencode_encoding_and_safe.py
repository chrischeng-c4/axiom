# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_encoding_and_safe"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: encoding= controls how str values are encoded before escaping (utf-8 default, latin-1, ascii+replace) and safe= leaves named chars unescaped in both keys and values"""
from urllib.parse import urlencode

pair = (("\xa0", "\xc1"),)
assert urlencode(pair) == "%C2%A0=%C3%81", "utf-8 default"
assert urlencode(pair, encoding="latin-1") == "%A0=%C1", "latin-1"
assert urlencode(pair, encoding="ASCII", errors="replace") == "%3F=%3F", "ascii replace"
assert urlencode(((b"\xa0$", b"\xc1$"),), safe=":$") == "%A0$=%C1$", "safe="

print("urlencode_encoding_and_safe OK")
