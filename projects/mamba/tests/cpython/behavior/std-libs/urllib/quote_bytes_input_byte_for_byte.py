# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_bytes_input_byte_for_byte"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: bytes input is escaped byte-for-byte; quote_from_bytes is the dedicated bytes path and agrees with quote"""
from urllib.parse import quote, quote_from_bytes

given = b"\xa2\xd8ab\xff"
assert quote(given) == "%A2%D8ab%FF", "quote(bytes)"
assert quote_from_bytes(given) == "%A2%D8ab%FF", "quote_from_bytes"

print("quote_bytes_input_byte_for_byte OK")
