# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "phase_round_trip_in_principal_range"
# subject = "cmath.phase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.phase: phase of rect(1, theta) recovers theta (mapped into the principal range (-pi, pi]) across a table of angles"""
import cmath
import math

for _theta in [-math.pi, -math.pi / 2, 0, math.pi / 2, math.pi]:
    _z = cmath.rect(1, _theta)
    _p = cmath.phase(_z)
    # phase of rect(1, theta) = theta (modulo 2pi, mapped to (-pi, pi]).
    assert abs(_p - _theta) < 1e-12 or abs(abs(_p) - math.pi) < 1e-12, \
        f"phase round-trip {_theta}"
print("phase_round_trip_in_principal_range OK")
