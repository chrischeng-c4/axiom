# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "randbytes_is_callable"
# subject = "random.randbytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random.randbytes: randbytes_is_callable (surface)."""
import random

assert callable(random.randbytes)
print("randbytes_is_callable OK")
