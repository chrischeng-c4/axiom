# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquote_bytes_input"
# subject = "urllib.parse.unquote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote: bytes input is decoded as UTF-8, whether plain ASCII bytes, raw UTF-8 bytes, or percent-escaped bytes"""
from urllib.parse import unquote

assert unquote(b"blueberryjam") == "blueberryjam", "ascii bytes"
assert unquote(b"bl\xc3\xa5b\xc3\xa6r") == "bl\xe5b\xe6r", "utf-8 bytes"
assert unquote(b"bl%c3%a5b%c3%a6r") == "bl\xe5b\xe6r", "percent-escaped bytes"

print("unquote_bytes_input OK")
