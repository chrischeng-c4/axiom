# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "translate_atomic_groups_prevent_backtracking"
# subject = "fnmatch.translate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.translate: translate rewrites interior '*literal' runs into atomic groups: '**a*a****a' -> (?s:(?>.*?a)(?>.*?a).*a)\\Z, and the undocumented paste-multiple-results feature still matches the intended strings"""
import re
import fnmatch

# Interior "*literal" runs become atomic groups (no catastrophic backtracking).
_t = fnmatch.translate("**a*a****a")
assert _t == "(?s:(?>.*?a)(?>.*?a).*a)\\Z", f"atomic groups = {_t!r}"

# Pasting multiple translate results is an undocumented feature that still
# matches the intended strings.
_fatre = "|".join([
    fnmatch.translate("**a**a**a*"),
    fnmatch.translate("**b**b**b*"),
    fnmatch.translate("*c*c*c*"),
])
assert re.match(_fatre, "abaccad"), "matches a-pattern"
assert re.match(_fatre, "abxbcab"), "matches b-pattern"
assert re.match(_fatre, "cbabcaxc"), "matches c-pattern"
assert not re.match(_fatre, "dabccbad"), "matches none"

print("translate_atomic_groups_prevent_backtracking OK")
