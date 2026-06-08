# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "fmod_fabs"
# subject = "math.fmod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.fmod: fmod keeps the dividend's sign (fmod(-10, 3)==-1.0, fmod(10, 3)==1.0) and fabs returns the absolute value as a float (fabs(-3.14)==3.14)"""
import math

assert math.fmod(10, 3) == 1.0, f"fmod(10,3) = {math.fmod(10, 3)!r}"
assert math.fmod(-10, 3) == -1.0, f"fmod(-10,3) = {math.fmod(-10, 3)!r}"
assert math.fabs(-3.14) == 3.14, f"fabs(-3.14) = {math.fabs(-3.14)!r}"
assert math.fabs(3.14) == 3.14, f"fabs(3.14) = {math.fabs(3.14)!r}"
assert isinstance(math.fabs(-3.14), float), "fabs returns float"

print("fmod_fabs OK")
