# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "security"
# case = "alternation_blowup_bounded"
# subject = "re.Pattern.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Pattern.search: overlapping-alternation evil-regex patterns (^(a|a)*$, ^(a|aa)+$, ^(ab|a|b)+$) on poison input must not match — and must not hang. The wall-clock budget is the harness sandbox timeout (ulimit -t), not a fixture-level timer."""
import re

N = 18

attacks = [
    (r"^(a|a)*$", "a" * N + "!"),
    (r"^(a|aa)+$", "a" * N + "!"),
    (r"^(ab|a|b)+$", "a" * N + "!"),
]
for pat, text in attacks:
    assert re.compile(pat).search(text) is None, f"expected no match for {pat!r} on poison input"

# Sanity: legitimate input still matches.
assert re.compile(r"^(a|aa)+$").search("aa" * 6) is not None, "alternation matches clean input"

print("alternation_blowup_bounded OK")
