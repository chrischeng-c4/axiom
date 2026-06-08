# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "special_constant_reprs"
# subject = "cmath.inf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.inf: repr of the special constants uses the compact spellings 'inf', 'infj', 'nan', 'nanj'"""
import cmath

assert repr(cmath.inf) == "inf", f"repr inf = {repr(cmath.inf)!r}"
assert repr(cmath.infj) == "infj", f"repr infj = {repr(cmath.infj)!r}"
assert repr(cmath.nan) == "nan", f"repr nan = {repr(cmath.nan)!r}"
assert repr(cmath.nanj) == "nanj", f"repr nanj = {repr(cmath.nanj)!r}"
print("special_constant_reprs OK")
