# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquote_utf8_and_latin1_decode"
# subject = "urllib.parse.unquote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote: unquote decodes %-escapes as UTF-8 by default and as latin-1 under encoding='latin-1'; errors=ignore drops un-decodable bytes and errors=replace yields U+FFFD"""
from urllib.parse import unquote

assert unquote("%E6%BC%A2%E5%AD%97") == "\u6f22\u5b57", "unquote utf-8"
assert unquote("br%C3%BCckner") == "br\xfcckner", "unquote utf-8 latin char"
assert unquote("br%FCckner", encoding="latin-1") == "br\xfcckner", "unquote latin-1"
assert unquote("%F3%B1", errors="ignore") == "", "unquote errors=ignore drops"
assert unquote("%F3%B1", errors="replace") == "\ufffd", "unquote errors=replace -> U+FFFD"

print("unquote_utf8_and_latin1_decode OK")
