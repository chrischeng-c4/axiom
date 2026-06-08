# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "choice_is_callable"
# subject = "random.choice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random.choice: choice_is_callable (surface)."""
import random

assert callable(random.choice)
print("choice_is_callable OK")
