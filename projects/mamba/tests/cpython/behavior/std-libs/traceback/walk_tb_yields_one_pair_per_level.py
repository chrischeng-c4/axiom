# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "walk_tb_yields_one_pair_per_level"
# subject = "traceback.walk_tb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.walk_tb: walk_tb yields one (frame, lineno) pair per traceback level: a single-frame ZeroDivisionError traceback has exactly one entry"""
import traceback

try:
    1 / 0
except ZeroDivisionError as e:
    _tb = e.__traceback__
assert len(list(traceback.walk_tb(_tb))) == 1, "walk_tb single-frame"

print("walk_tb_yields_one_pair_per_level OK")
