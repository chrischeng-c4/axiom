# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "htmldiff_default_charset_utf8"
# subject = "difflib.HtmlDiff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.HtmlDiff: HtmlDiff().make_file defaults the <meta> charset to utf-8"""
import difflib

_from = ["Beautiful is better than ugly", "Explicit is better"]
_to = ["Beautiful is better than nice", "Explicit is best"]
_default = difflib.HtmlDiff().make_file(_from, _to)
assert 'content="text/html; charset=utf-8"' in _default, "default charset utf-8"
print("htmldiff_default_charset_utf8 OK")
