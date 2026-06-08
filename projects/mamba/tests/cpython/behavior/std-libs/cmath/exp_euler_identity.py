# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "exp_euler_identity"
# subject = "cmath.exp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.exp: Euler's identity: exp(j*pi) + 1 has magnitude ~ 0"""
import cmath
import math

_euler = cmath.exp(1j * math.pi) + 1
assert abs(_euler) < 1e-12, f"|exp(j*pi)+1| = {abs(_euler)!r}"
print("exp_euler_identity OK")
