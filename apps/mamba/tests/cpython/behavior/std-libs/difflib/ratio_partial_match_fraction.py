# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "ratio_partial_match_fraction"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.SequenceMatcher: ratio() of 'hello' vs 'helo' is 2*M/T = 8/9; quick_ratio is an upper bound >= ratio"""
import difflib

_sm = difflib.SequenceMatcher(None, "hello", "helo")
_r = _sm.ratio()
# 4 matched chars out of (5 + 4) total -> 2*4/9
assert _r == 2 * 4 / (5 + 4), f"ratio = {_r!r}"
assert 0.0 <= _r <= 1.0, f"ratio range = {_r!r}"
assert _r > 0.5, f"ratio > 0.5 = {_r!r}"
# quick_ratio is an upper bound on ratio.
_qr = _sm.quick_ratio()
assert _qr >= _r, f"quick_ratio {_qr!r} >= ratio {_r!r}"
print("ratio_partial_match_fraction OK")
