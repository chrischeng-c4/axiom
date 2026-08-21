# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "phase_signed_zero_real_axis"
# subject = "cmath.phase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.phase: phase() on the real axis respects signed zero: phase(+0+0j)=0, (+0-0j)=-0, (-0+0j)=pi, (-0-0j)=-pi"""
import cmath
import math

assert cmath.phase(complex(0.0, 0.0)) == 0.0, "phase(+0+0j) = 0"
# Imaginary -0 picks the negative branch even on the positive real axis.
assert cmath.phase(complex(0.0, -0.0)) == -0.0, "phase(+0-0j) = -0"
assert cmath.phase(complex(-0.0, 0.0)) == math.pi, "phase(-0+0j) = pi"
assert cmath.phase(complex(-0.0, -0.0)) == -math.pi, "phase(-0-0j) = -pi"
print("phase_signed_zero_real_axis OK")
