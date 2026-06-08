# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "process_time_non_negative_float"
# subject = "time.process_time"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.process_time: time.process_time() returns a non-negative float"""
import time

_pt = time.process_time()
assert isinstance(_pt, float), f"process_time type = {type(_pt)!r}"
assert _pt >= 0, f"process_time >= 0: {_pt!r}"
print("process_time_non_negative_float OK")
