# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquote_plus_handling"
# subject = "urllib.parse.unquote_plus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote_plus: unquote leaves '+' alone while unquote_plus turns '+' into a space; a valid escape embedded in plain text still decodes"""
from urllib.parse import unquote, unquote_plus

assert unquote("are+there+spaces") == "are+there+spaces", "unquote keeps +"
assert unquote_plus("are+there+spaces") == "are there spaces", "unquote_plus + -> space"
assert unquote("ab%63d") == "abcd", "embedded escape"

print("unquote_plus_handling OK")
