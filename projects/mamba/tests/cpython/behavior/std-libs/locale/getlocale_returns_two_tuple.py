# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "getlocale_returns_two_tuple"
# subject = "locale.getlocale"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.getlocale: getlocale() returns a (language, encoding) 2-tuple"""
import locale

loc = locale.getlocale()
assert isinstance(loc, tuple), "getlocale -> tuple"
assert len(loc) == 2, "getlocale -> 2-tuple"

print("getlocale_returns_two_tuple OK")
