# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "system_random_class_is_callable"
# subject = "random.SystemRandom"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random.SystemRandom: system_random_class_is_callable (surface)."""
import random

assert callable(random.SystemRandom)
print("system_random_class_is_callable OK")
