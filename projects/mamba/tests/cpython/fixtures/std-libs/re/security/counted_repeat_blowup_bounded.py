# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "security"
# case = "counted_repeat_blowup_bounded"
# subject = "re.Pattern.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Pattern.search: counted-repeat blowup patterns ((.*a){12}$, (a?){18}a{18}) on hostile input finish under a 1.0s budget and the engine still finds legitimate matches"""
import re

import time

BUDGET = 1.0

attacks = [
    (r"(.*a){12}$", "a" * 11 + "!"),
    (r"(a?){18}a{18}", "a" * 17),
]
slowest = 0.0
for pat, text in attacks:
    rx = re.compile(pat)
    t0 = time.perf_counter()
    m = rx.search(text)
    elapsed = time.perf_counter() - t0
    assert elapsed < BUDGET, f"counted-repeat ReDoS budget blown by {pat!r}: {elapsed:.3f}s"
    assert m is None, f"expected no match for {pat!r} on poison input"
    slowest = max(slowest, elapsed)
assert slowest < BUDGET, "all counted-repeat attacks under budget"

# Sanity: legitimate input still matches quickly.
assert re.compile(r"(a?){18}a{18}").search("a" * 18) is not None, "counted repeat matches clean input"

print("counted_repeat_blowup_bounded OK")
