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
"""re.Pattern.search: overlapping-alternation evil-regex patterns (^(a|a)*$, ^(a|aa)+$, ^(ab|a|b)+$) on poison input finish under a 1.0s budget and still match clean legitimate input quickly"""
import re

import time

N = 18
BUDGET = 1.0

attacks = [
    (r"^(a|a)*$", "a" * N + "!"),
    (r"^(a|aa)+$", "a" * N + "!"),
    (r"^(ab|a|b)+$", "a" * N + "!"),
]
slowest = 0.0
for pat, text in attacks:
    rx = re.compile(pat)
    t0 = time.perf_counter()
    m = rx.search(text)
    elapsed = time.perf_counter() - t0
    assert elapsed < BUDGET, f"alternation ReDoS budget blown by {pat!r}: {elapsed:.3f}s"
    assert m is None, f"expected no match for {pat!r} on poison input"
    slowest = max(slowest, elapsed)
assert slowest < BUDGET, "all alternation attacks under budget"

# Sanity: legitimate input still matches quickly.
assert re.compile(r"^(a|aa)+$").search("aa" * 6) is not None, "alternation matches clean input"

print("alternation_blowup_bounded OK")
