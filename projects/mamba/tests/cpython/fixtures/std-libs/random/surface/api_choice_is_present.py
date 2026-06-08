# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "api_choice_is_present"
# subject = "random.choice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""random.choice: api_choice_is_present (surface)."""
import random

assert hasattr(random, "choice")
print("api_choice_is_present OK")
