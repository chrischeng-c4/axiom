# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "float_forms_and_special"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: floats parse plain (3.14159), negative, exponent (6.022e23, 1e-10), and the special values inf and nan"""
import tomllib
import math

_d = tomllib.loads("""
pi = 3.14159
neg = -2.5
exp = 6.022e23
small = 1e-10
inf = inf
nan = nan
""")
assert abs(_d["pi"] - 3.14159) < 1e-10, f"pi = {_d['pi']!r}"
assert _d["neg"] == -2.5, f"neg = {_d['neg']!r}"
assert abs(_d["exp"] - 6.022e23) / 6.022e23 < 1e-10, f"exp = {_d['exp']!r}"
assert abs(_d["small"] - 1e-10) < 1e-20, f"small = {_d['small']!r}"
assert math.isinf(_d["inf"]) and _d["inf"] > 0, f"inf = {_d['inf']!r}"
assert math.isnan(_d["nan"]), f"nan = {_d['nan']!r}"

print("float_forms_and_special OK")
