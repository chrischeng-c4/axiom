# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "nanj_attr"
# subject = "cmath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath: nanj_attr (surface)."""
import cmath

assert hasattr(cmath, "nanj")
print("nanj_attr OK")
