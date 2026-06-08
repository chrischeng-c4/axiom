# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "surface"
# case = "api_fraction_is_present"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fractions.Fraction: api_fraction_is_present (surface)."""
import fractions

assert hasattr(fractions, "Fraction")
print("api_fraction_is_present OK")
