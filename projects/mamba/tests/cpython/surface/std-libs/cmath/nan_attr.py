# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "nan_attr"
# subject = "cmath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath: nan_attr (surface)."""
import cmath

assert hasattr(cmath, "nan")
print("nan_attr OK")
