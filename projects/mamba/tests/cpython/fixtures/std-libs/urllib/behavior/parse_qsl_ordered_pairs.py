# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "parse_qsl_ordered_pairs"
# subject = "urllib.parse.parse_qsl"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.parse_qsl: parse_qsl returns an ordered list of (key, value) tuples preserving input order and duplicate keys"""
from urllib.parse import parse_qsl

assert parse_qsl("a=1&b=2&a=3") == [("a", "1"), ("b", "2"), ("a", "3")], "ordered dup"
assert parse_qsl("k1=v1&k2=v2") == [("k1", "v1"), ("k2", "v2")], "simple"

print("parse_qsl_ordered_pairs OK")
