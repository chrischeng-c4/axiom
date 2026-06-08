# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "getpid_is_positive_int"
# subject = "os.getpid"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.getpid: os.getpid returns a positive int (the live interpreter PID)"""
import os

pid = os.getpid()
assert isinstance(pid, int), f"pid type = {type(pid)!r}"
assert pid > 0, f"pid > 0: {pid!r}"
print("getpid_is_positive_int OK")
