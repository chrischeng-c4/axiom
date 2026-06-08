# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_plus_space_to_plus"
# subject = "urllib.parse.quote_plus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote_plus: quote_plus encodes each space as '+' (the form-encoding convention)"""
from urllib.parse import quote_plus

assert quote_plus("hello world") == "hello+world", repr(quote_plus("hello world"))
assert quote_plus("a b c") == "a+b+c", repr(quote_plus("a b c"))

print("quote_plus_space_to_plus OK")
