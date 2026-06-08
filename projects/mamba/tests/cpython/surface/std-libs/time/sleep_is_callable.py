# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "sleep_is_callable"
# subject = "time.sleep"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.sleep: sleep_is_callable (surface)."""
import time

assert callable(time.sleep)
print("sleep_is_callable OK")
