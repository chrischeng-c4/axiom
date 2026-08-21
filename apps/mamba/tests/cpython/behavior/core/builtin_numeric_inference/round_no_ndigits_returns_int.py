# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "round_no_ndigits_returns_int"
# subject = "round(float) with no ndigits returns int, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""round(2.567) with no ndigits must return the int 3 (banker's rounding) when assigned then used."""

r = round(2.567)
assert r == 3, r
assert isinstance(r, int), type(r)
# banker's rounding: round(2.5) is 2, round(3.5) is 4
assert round(2.5) == 2, round(2.5)
assert round(3.5) == 4, round(3.5)
nxt = r + 1
assert nxt == 4, nxt
print("round_no_ndigits_returns_int OK")
