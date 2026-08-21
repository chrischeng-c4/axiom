# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "behavior"
# case = "gmtime_epoch_tuple_slice"
# subject = "time.gmtime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.gmtime: tuple(time.gmtime(0))[:6] equals (1970, 1, 1, 0, 0, 0) — struct_time is tuple-like by index and by slice"""
import time

assert tuple(time.gmtime(0))[:6] == (1970, 1, 1, 0, 0, 0), "epoch tuple slice"
print("gmtime_epoch_tuple_slice OK")
