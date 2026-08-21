# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "trig_identities"
# subject = "math.sin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sin: core trig identities within 1e-10: sin(pi)~=0, cos(pi)==-1, tan(pi/4)==1, sin(0)==0, cos(0)==1, sin(pi/2)==1"""
import math

_eps = 1e-10
assert abs(math.sin(math.pi) - 0.0) < _eps, f"sin(pi) = {math.sin(math.pi)!r}"
assert abs(math.cos(math.pi) - (-1.0)) < _eps, f"cos(pi) = {math.cos(math.pi)!r}"
assert abs(math.tan(math.pi / 4) - 1.0) < _eps, f"tan(pi/4) = {math.tan(math.pi / 4)!r}"
assert abs(math.sin(0)) < _eps, f"sin(0) = {math.sin(0)!r}"
assert abs(math.cos(0) - 1.0) < _eps, f"cos(0) = {math.cos(0)!r}"
assert abs(math.sin(math.pi / 2) - 1.0) < _eps, f"sin(pi/2) = {math.sin(math.pi / 2)!r}"

print("trig_identities OK")
