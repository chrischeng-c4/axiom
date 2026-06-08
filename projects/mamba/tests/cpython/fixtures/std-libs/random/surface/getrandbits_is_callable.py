# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "getrandbits_is_callable"
# subject = "random.getrandbits"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random.getrandbits: getrandbits_is_callable (surface)."""
import random

assert callable(random.getrandbits)
print("getrandbits_is_callable OK")
