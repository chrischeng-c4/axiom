# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "divmod_float_assign"
# subject = "divmod(float, float) tuple unpack, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""divmod(7.5, 2.0) unpacked into variables must yield the correct float quotient and remainder."""

q, r = divmod(7.5, 2.0)
assert q == 3.0, q
assert r == 1.5, r
assert isinstance(q, float), type(q)
assert isinstance(r, float), type(r)
total = q * 2.0 + r
assert total == 7.5, total
print("divmod_float_assign OK")
