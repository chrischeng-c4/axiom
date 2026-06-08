# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "parse_qs_dict_of_lists"
# subject = "urllib.parse.parse_qs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.parse_qs: parse_qs returns a dict mapping each key to the list of its values, grouping repeated keys"""
from urllib.parse import parse_qs

qs = parse_qs("a=1&b=2&a=3")
assert sorted(qs.keys()) == ["a", "b"], f"keys = {sorted(qs.keys())!r}"
assert "1" in qs["a"] and "3" in qs["a"], f"a vals = {qs['a']!r}"
assert qs["b"] == ["2"], f"b = {qs['b']!r}"

print("parse_qs_dict_of_lists OK")
