# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "cpu_count_is_callable"
# subject = "os.cpu_count"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.cpu_count: cpu_count_is_callable (surface)."""
import os

assert callable(os.cpu_count)
print("cpu_count_is_callable OK")
