# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "strcoll_orders_like_c"
# subject = "locale.strcoll"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.strcoll: strcoll returns <0, 0, >0 for a<b, a==a, b>a in the C locale (byte ordering)"""
import locale

# In the default 'C' locale strcoll reduces to byte ordering and is
# deterministic, so no locale-gating is needed.
assert locale.strcoll("a", "b") < 0, "strcoll a<b"
assert locale.strcoll("a", "a") == 0, "strcoll a==a"
assert locale.strcoll("b", "a") > 0, "strcoll b>a"

print("strcoll_orders_like_c OK")
