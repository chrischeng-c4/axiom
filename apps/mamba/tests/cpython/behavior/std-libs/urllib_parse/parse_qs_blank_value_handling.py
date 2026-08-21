# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "parse_qs_blank_value_handling"
# subject = "urllib.parse.parse_qs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.parse_qs: parse_qs drops blank values by default but keeps them (as ['']) under keep_blank_values=True; repeated keys collect into a list and parse_qsl returns ordered (key, value) pairs"""
from urllib.parse import parse_qs, parse_qsl

qs = parse_qs("a=1&b=&c=3")
assert "b" not in qs, f"blank value excluded by default = {qs!r}"
qs_blank = parse_qs("a=1&b=&c=3", keep_blank_values=True)
assert qs_blank["b"] == [""], f"blank value kept = {qs_blank['b']!r}"

multi = parse_qs("a=1&b=2&a=3")
assert multi["a"] == ["1", "3"], f"repeated key collects = {multi['a']!r}"
assert multi["b"] == ["2"], f"single value = {multi['b']!r}"

pairs = parse_qsl("a=1&b=2&a=3")
assert pairs == [("a", "1"), ("b", "2"), ("a", "3")], f"parse_qsl ordered = {pairs!r}"

print("parse_qs_blank_value_handling OK")
