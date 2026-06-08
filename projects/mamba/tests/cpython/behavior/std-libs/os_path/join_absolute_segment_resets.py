# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "join_absolute_segment_resets"
# subject = "os.path.join"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.join: an absolute (leading-slash) segment discards everything before it; join('a','b','/abs','c') == '/abs/c' and join('/root','/other') == '/other'"""
import os.path

assert os.path.join("a", "/abs") == "/abs", "absolute segment resets"
assert os.path.join("a", "b", "/abs", "c") == "/abs/c", "abs resets mid-list"
assert os.path.join("/root", "/other") == "/other", "abs-abs reset"

print("join_absolute_segment_resets OK")
