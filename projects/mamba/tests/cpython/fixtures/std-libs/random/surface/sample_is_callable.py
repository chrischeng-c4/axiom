# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "sample_is_callable"
# subject = "random.sample"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random.sample: sample_is_callable (surface)."""
import random

assert callable(random.sample)
print("sample_is_callable OK")
