# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "html_parser"
# dimension = "errors"
# case = "feed_int_raises_typeerror"
# subject = "html.parser.HTMLParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""html.parser.HTMLParser: feed_int_raises_typeerror (errors)."""
from html.parser import HTMLParser

_raised = False
try:
    HTMLParser().feed(123)
except TypeError:
    _raised = True
assert _raised, "feed_int_raises_typeerror: expected TypeError"
print("feed_int_raises_typeerror OK")
