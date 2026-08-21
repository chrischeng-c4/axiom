# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "shuffle_is_callable"
# subject = "random.shuffle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random.shuffle: shuffle_is_callable (surface)."""
import random

assert callable(random.shuffle)
print("shuffle_is_callable OK")
