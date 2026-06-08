# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "seed_is_callable"
# subject = "random.seed"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random.seed: seed_is_callable (surface)."""
import random

assert callable(random.seed)
print("seed_is_callable OK")
