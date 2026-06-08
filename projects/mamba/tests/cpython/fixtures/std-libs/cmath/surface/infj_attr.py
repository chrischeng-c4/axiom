# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "infj_attr"
# subject = "cmath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath: infj_attr (surface)."""
import cmath

assert hasattr(cmath, "infj")
print("infj_attr OK")
