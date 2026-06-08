# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "monotonic_is_callable"
# subject = "time.monotonic"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.monotonic: monotonic_is_callable (surface)."""
import time

assert callable(time.monotonic)
print("monotonic_is_callable OK")
