# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "real_world"
# case = "ac_circuit_impedance"
# subject = "cmath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath: model an AC series RLC circuit: build complex impedance Z = R + j(wL - 1/(wC)), recover its magnitude and phase with abs()/cmath.phase(), and confirm polar/rect round-trips Z"""
import cmath
import math

# Series RLC branch driven at angular frequency omega.
R = 50.0            # ohms
L = 10e-3           # henries
C = 100e-6          # farads
omega = 2 * math.pi * 60.0  # 60 Hz mains

# Z = R + j(wL - 1/(wC)) — resistance plus net reactance.
reactance = omega * L - 1.0 / (omega * C)
Z = complex(R, reactance)

# Magnitude (|Z|) and phase angle recovered from the complex impedance.
magnitude = abs(Z)
angle = cmath.phase(Z)

# |Z| = sqrt(R^2 + X^2); the resistor is below the total impedance.
assert abs(magnitude - math.hypot(R, reactance)) < 1e-9, magnitude
assert magnitude >= R, "total impedance is at least the resistance"

# Capacitive branch (1/wC dominates wL here) -> negative reactance -> phase < 0.
assert reactance < 0.0, reactance
assert -math.pi / 2 < angle < 0.0, angle

# polar() exposes the same (magnitude, angle), and rect() rebuilds Z exactly.
r, phi = cmath.polar(Z)
assert abs(r - magnitude) < 1e-9, (r, magnitude)
assert abs(phi - angle) < 1e-12, (phi, angle)
Z_back = cmath.rect(r, phi)
assert abs(Z_back - Z) < 1e-9, (Z_back, Z)

print("ac_circuit_impedance OK")
