# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "twopi_constant_attr"
# subject = "random"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random: twopi_constant_attr (surface)."""
import random

assert hasattr(random, "TWOPI")
print("twopi_constant_attr OK")
