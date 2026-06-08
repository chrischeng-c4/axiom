# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "ascii_error_handlers"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: ascii error handlers on 'café': 'replace' -> b'caf?', 'ignore' -> b'caf', 'xmlcharrefreplace' inserts b'&#233;'"""
import codecs

_replace = codecs.encode("café", "ascii", "replace")
assert _replace == b"caf?", f"replace = {_replace!r}"
_ignore = codecs.encode("café", "ascii", "ignore")
assert _ignore == b"caf", f"ignore = {_ignore!r}"
_xml = codecs.encode("café", "ascii", "xmlcharrefreplace")
assert b"&#233;" in _xml, f"xmlcharrefreplace = {_xml!r}"

print("ascii_error_handlers OK")
