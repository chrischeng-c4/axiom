# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "htmldiff_nonascii_escaped_under_usascii"
# subject = "difflib.HtmlDiff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.HtmlDiff: non-ASCII input under charset='us-ascii' is escaped to numeric character references (e.g. &#305;)"""
import difflib

_nonascii_from = ["Explicit is better than ımplıcıt"]
_nonascii_to = ["Explicit is better than implicit"]
_usascii = difflib.HtmlDiff().make_file(
    _nonascii_from, _nonascii_to, charset="us-ascii")
assert 'content="text/html; charset=us-ascii"' in _usascii, "us-ascii charset"
assert "&#305;" in _usascii, "non-ascii escaped to numeric entity (&#305;)"
print("htmldiff_nonascii_escaped_under_usascii OK")
