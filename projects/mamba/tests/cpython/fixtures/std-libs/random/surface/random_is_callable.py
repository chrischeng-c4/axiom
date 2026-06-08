# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "random_is_callable"
# subject = "random.random"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random.random: random_is_callable (surface)."""
import random

assert callable(random.random)
print("random_is_callable OK")
