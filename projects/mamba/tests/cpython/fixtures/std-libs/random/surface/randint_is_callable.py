# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "randint_is_callable"
# subject = "random.randint"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random.randint: randint_is_callable (surface)."""
import random

assert callable(random.randint)
print("randint_is_callable OK")
