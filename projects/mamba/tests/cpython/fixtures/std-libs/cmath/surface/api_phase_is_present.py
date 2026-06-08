# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_phase_is_present"
# subject = "cmath.phase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.phase: api_phase_is_present (surface)."""
import cmath

assert hasattr(cmath, "phase")
print("api_phase_is_present OK")
