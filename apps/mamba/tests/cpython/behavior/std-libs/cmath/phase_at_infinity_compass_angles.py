# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "phase_at_infinity_compass_angles"
# subject = "cmath.phase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.phase: phase() of directions at infinity collapses to the compass angles (pi/4, 3pi/4, etc.); a finite component is dwarfed by an infinite one"""
import cmath
import math

INF = float("inf")

assert abs(cmath.phase(complex(INF, INF)) - math.pi / 4) < 1e-9, "phase NE = pi/4"
assert abs(cmath.phase(complex(-INF, INF)) - 0.75 * math.pi) < 1e-9, "phase NW"
assert abs(cmath.phase(complex(-INF, -INF)) + 0.75 * math.pi) < 1e-9, "phase SW"
assert abs(cmath.phase(complex(INF, -INF)) + math.pi / 4) < 1e-9, "phase SE"
# A finite component is dwarfed by an infinite one.
assert abs(cmath.phase(complex(2.3, INF)) - math.pi / 2) < 1e-9, "phase up = pi/2"
assert cmath.phase(complex(INF, 2.3)) == 0.0, "phase right = 0"
assert abs(cmath.phase(complex(-INF, 2.3)) - math.pi) < 1e-9, "phase left = pi"
print("phase_at_infinity_compass_angles OK")
