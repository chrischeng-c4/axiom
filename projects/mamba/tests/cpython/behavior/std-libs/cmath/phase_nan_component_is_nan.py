# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "phase_nan_component_is_nan"
# subject = "cmath.phase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.phase: phase() is NaN when any component is NaN"""
import cmath
import math

NAN = float("nan")

assert math.isnan(cmath.phase(complex(NAN, 1.0))), "phase(nan+1j) is nan"
assert math.isnan(cmath.phase(complex(1.0, NAN))), "phase(1+nanj) is nan"
assert math.isnan(cmath.phase(complex(NAN, NAN))), "phase(nan+nanj) is nan"
print("phase_nan_component_is_nan OK")
