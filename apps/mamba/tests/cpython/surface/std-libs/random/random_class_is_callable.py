# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "random_class_is_callable"
# subject = "random.Random"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random.Random: random_class_is_callable (surface)."""
import random

assert callable(random.Random)
print("random_class_is_callable OK")
