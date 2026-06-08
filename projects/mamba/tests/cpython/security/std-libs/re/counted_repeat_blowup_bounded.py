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
"""re.Pattern.search: counted-repeat blowup patterns ((.*a){12}$, (a?){18}a{18}) on hostile input must not match — and must not hang. The wall-clock budget is the harness sandbox timeout (ulimit -t), not a fixture-level timer."""
import re

attacks = [
    (r"(.*a){12}$", "a" * 11 + "!"),
    (r"(a?){18}a{18}", "a" * 17),
]
for pat, text in attacks:
    assert re.compile(pat).search(text) is None, f"expected no match for {pat!r} on poison input"

# Sanity: legitimate input still matches.
assert re.compile(r"(a?){18}a{18}").search("a" * 18) is not None, "counted repeat matches clean input"

print("counted_repeat_blowup_bounded OK")
