# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "round_literal_ndigits_assign"
# subject = "round(float, ndigits) on literals, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""round(2.567, 2) assigned to a variable must yield 2.57 as a float."""

r = round(2.567, 2)
assert r == 2.57, r
assert isinstance(r, float), type(r)
scaled = r * 100.0
assert scaled == 257.0, scaled
print("round_literal_ndigits_assign OK")
