# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "active_count_includes_main"
# subject = "threading.active_count"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.active_count: active_count() always includes the main thread, so it is >= 1"""
import threading

_cnt = threading.active_count()
assert isinstance(_cnt, int), f"active_count type = {type(_cnt)!r}"
assert _cnt >= 1, f"active_count = {_cnt!r}"

print("active_count_includes_main OK")
