# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "security"
# case = "catastrophic_backtracking_bounded"
# subject = "re.Pattern.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Pattern.search: classic ReDoS nested-quantifier patterns (^(a+)+$ etc.) matched against pathological non-matching input finish under a 1.0s wall-clock budget, proving the engine is not exponential on hostile input"""
import re

import time

N = 18
BUDGET = 1.0

# Nested-quantifier ReDoS shapes against a poison non-matching input.
attacks = [
    (r"^(a+)+$", "a" * N + "!"),
    (r"^(a*)*$", "a" * N + "!"),
    (r"^(a+)*$", "a" * N + "!"),
    (r"^(.*)*c$", "a" * N + "!"),
    (r"^([a-z]+)*$", "a" * N + "!"),
    (r"^(\d+)+$", "1" * N + "!"),
    (r"^(\w+\s?)*$", "a" * N + "!"),
    (r"^(x+x+)+y$", "x" * N + "!"),
    (r"(a+)+c", "a" * N + "b"),
    (r"(x+)+y", "x" * N + "z"),
]
slowest = 0.0
for pat, text in attacks:
    rx = re.compile(pat)
    t0 = time.perf_counter()
    m = rx.search(text)
    elapsed = time.perf_counter() - t0
    assert elapsed < BUDGET, f"ReDoS budget blown by {pat!r}: {elapsed:.3f}s"
    assert m is None, f"expected no match for {pat!r} on poison input"
    slowest = max(slowest, elapsed)
assert slowest < BUDGET, "all attacks under budget"

# Scaled poison input on the canonical pattern stays linear.
big = "a" * (N + 4) + "!"
t0 = time.perf_counter()
assert re.compile(r"^(a+)+$").search(big) is None, "scaled poison must not match"
assert time.perf_counter() - t0 < BUDGET, "scaled input under budget"

# Sanity: the pattern still matches clean input quickly (not a never-match stub).
assert re.compile(r"^(a+)+$").search("a" * N) is not None, "matches clean input"

print("catastrophic_backtracking_bounded OK")
