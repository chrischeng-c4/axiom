# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "matching_blocks_cached_and_sentinel"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: get_matching_blocks for 'abxcd' vs 'abcd' is cached/idempotent; block[0] is Match(a=0,b=0,size=2) and the trailing sentinel block has size 0"""
import difflib

_sm = difflib.SequenceMatcher(None, "abxcd", "abcd")
_first = _sm.get_matching_blocks()
_second = _sm.get_matching_blocks()
assert _first == _second, "matching blocks cached/idempotent"
assert _second[0].a == 0 and _second[0].b == 0, f"block[0] a/b = {_second[0]!r}"
assert _second[0].size == 2, f"block[0].size = {_second[0].size!r}"
assert _second[-1].size == 0, "sentinel block has size 0"
print("matching_blocks_cached_and_sentinel OK")
