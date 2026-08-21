# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "randrange_is_callable"
# subject = "random.randrange"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random.randrange: randrange_is_callable (surface)."""
import random

assert callable(random.randrange)
print("randrange_is_callable OK")
