# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "uniform_is_callable"
# subject = "random.uniform"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random.uniform: uniform_is_callable (surface)."""
import random

assert callable(random.uniform)
print("uniform_is_callable OK")
