# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "getitem_index_and_name"
# subject = "re.Match.__getitem__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Match.__getitem__: Match supports m[index] and m[name]; unmatched optional/alternation groups give None"""
import re

pat = re.compile(r"(?:(?P<a1>a)|(?P<b2>b))(?P<c3>c)?")
m = pat.match("a")
assert m[0] == "a", f"m[0] = {m[0]!r}"
assert m[1] == "a", f"m[1] = {m[1]!r}"
assert m[2] is None, "unmatched alternation group -> None"
assert m["a1"] == "a", "getitem by name"
assert m["b2"] is None, "unmatched named group -> None"
assert m["c3"] is None, "unmatched optional -> None"

print("getitem_index_and_name OK")
