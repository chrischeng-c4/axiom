# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "htmldiff_explicit_charset"
# subject = "difflib.HtmlDiff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.HtmlDiff: an explicit charset='iso-8859-1' flows into the make_file <meta> tag"""
import difflib

_from = ["Beautiful is better than ugly", "Explicit is better"]
_to = ["Beautiful is better than nice", "Explicit is best"]
_iso = difflib.HtmlDiff().make_file(_from, _to, charset="iso-8859-1")
assert 'content="text/html; charset=iso-8859-1"' in _iso, "iso-8859-1 charset"
print("htmldiff_explicit_charset OK")
