# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "filter_positional_defaults"
# subject = "tracemalloc.Filter"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Filter class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Filter: Filter(True, 'abc') defaults lineno to None and all_frames to False and exposes inclusive/filename_pattern"""
import tracemalloc

# Positional construction with defaults for lineno / all_frames.
f = tracemalloc.Filter(True, "abc")
assert f.inclusive is True, "inclusive"
assert f.filename_pattern == "abc", "filename_pattern"
assert f.lineno is None, "lineno default None"
assert f.all_frames is False, "all_frames default False"

print("filter_positional_defaults OK")
