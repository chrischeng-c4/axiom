# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "degrees_radians_inverse"
# subject = "math.degrees"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.degrees: degrees and radians are inverses: degrees(pi)==180.0 and radians(180)==pi within 1e-10"""
import math

_eps = 1e-10
assert abs(math.degrees(math.pi) - 180.0) < _eps, f"degrees(pi) = {math.degrees(math.pi)!r}"
assert abs(math.radians(180) - math.pi) < _eps, f"radians(180) = {math.radians(180)!r}"

print("degrees_radians_inverse OK")
