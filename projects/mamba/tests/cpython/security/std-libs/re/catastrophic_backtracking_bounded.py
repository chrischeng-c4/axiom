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
"""re.Pattern.search: classic ReDoS nested-quantifier patterns (^(a+)+$ etc.) on pathological non-matching input must not match — and must not hang. The wall-clock budget is the harness sandbox timeout (ulimit -t): an exponential engine is killed there, not timed inside the fixture."""
import re

N = 18

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
for pat, text in attacks:
    assert re.compile(pat).search(text) is None, f"expected no match for {pat!r} on poison input"

# Scaled poison input on the canonical pattern still must not match.
big = "a" * (N + 4) + "!"
assert re.compile(r"^(a+)+$").search(big) is None, "scaled poison must not match"

# Sanity: the pattern still matches clean input (not a never-match stub).
assert re.compile(r"^(a+)+$").search("a" * N) is not None, "matches clean input"

print("catastrophic_backtracking_bounded OK")
