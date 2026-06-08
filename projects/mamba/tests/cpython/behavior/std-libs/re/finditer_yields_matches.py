# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "finditer_yields_matches"
# subject = "re.Pattern.finditer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Pattern.finditer: finditer yields Match objects in order, and honors the optional pos/endpos window: re.compile(r':+').finditer over 'a:b::c:::d' all vs windowed (3,8)"""
import re

fp = re.compile(r":+")
assert [x.group() for x in fp.finditer("a:b::c:::d")] == [":", "::", ":::"], "finditer all"
assert [x.group() for x in fp.finditer("a:b::c:::d", 3, 8)] == ["::", "::"], "finditer window"

print("finditer_yields_matches OK")
