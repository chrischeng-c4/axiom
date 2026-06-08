# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "inf_lives_on_real_axis"
# subject = "cmath.inf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.inf: cmath.inf has +inf real part and 0 imaginary part; infj is the imaginary-axis mirror"""
import cmath
import math

assert cmath.inf.real == math.inf, f"inf.real = {cmath.inf.real!r}"
assert cmath.inf.imag == 0.0, f"inf.imag = {cmath.inf.imag!r}"
assert cmath.infj.real == 0.0, f"infj.real = {cmath.infj.real!r}"
assert cmath.infj.imag == math.inf, f"infj.imag = {cmath.infj.imag!r}"
print("inf_lives_on_real_axis OK")
