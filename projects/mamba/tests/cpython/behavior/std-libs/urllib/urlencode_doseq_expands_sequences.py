# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_doseq_expands_sequences"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: without doseq a list value is str()-ed whole; with doseq each element becomes its own key=value pair and a mapping value iterates its keys"""
from urllib.parse import urlencode, quote_plus

import collections

seq = {"sequence": ["1", "2", "3"]}
assert urlencode(seq) == "sequence=" + quote_plus(str(["1", "2", "3"])), \
    "no-doseq stringifies list"
expanded = urlencode(seq, doseq=True)
assert expanded.count("&") == 2, f"doseq count = {expanded!r}"
for v in ("sequence=1", "sequence=2", "sequence=3"):
    assert v in expanded, f"{v} in {expanded!r}"
assert urlencode({"a": [1, 2]}, True) == "a=1&a=2", "doseq ints"
assert urlencode({"a": [None, "a"]}, True) == "a=None&a=a", "doseq None"
od = collections.OrderedDict([("a", 1), ("b", 1)])
assert urlencode({"a": od}, True) == "a=a&a=b", "doseq over mapping keys"

print("urlencode_doseq_expands_sequences OK")
