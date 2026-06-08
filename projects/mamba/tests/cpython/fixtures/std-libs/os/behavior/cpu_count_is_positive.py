# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "cpu_count_is_positive"
# subject = "os.cpu_count"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.cpu_count: os.cpu_count returns a positive int (number of logical CPUs)"""
import os

count = os.cpu_count()
assert count is not None, "cpu_count is not None"
assert isinstance(count, int), f"cpu_count type = {type(count)!r}"
assert count > 0, f"cpu_count > 0: {count!r}"
print("cpu_count_is_positive OK")
