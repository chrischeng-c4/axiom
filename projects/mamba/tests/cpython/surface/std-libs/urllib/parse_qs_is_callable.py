# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "parse_qs_is_callable"
# subject = "urllib.parse.parse_qs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.parse_qs: parse_qs_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.parse_qs)
print("parse_qs_is_callable OK")
