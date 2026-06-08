# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "inf_attr"
# subject = "cmath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath: inf_attr (surface)."""
import cmath

assert hasattr(cmath, "inf")
print("inf_attr OK")
