# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "islice_stop_start_step_forms"
# subject = "itertools.islice"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.islice: islice supports stop-only, start/stop, and start/stop/step; over-long stop is clamped to the input"""
import itertools

assert list(itertools.islice(range(10), 3)) == [0, 1, 2], "stop only"
assert list(itertools.islice(range(20), 2, 8)) == [2, 3, 4, 5, 6, 7], "start/stop"
assert list(itertools.islice(range(10), 2, 7, 2)) == [2, 4, 6], "start/stop/step"
assert list(itertools.islice(range(5), 100)) == [0, 1, 2, 3, 4], "stop past end clamps"
assert list(itertools.islice("abcdef", 2, 5)) == ["c", "d", "e"], "string slice"

print("islice_stop_start_step_forms OK")
