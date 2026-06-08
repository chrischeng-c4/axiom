# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "exists_nonstring_returns_false_no_raise"
# subject = "os.path.exists"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.exists: exists is forgiving on a non-path argument: exists(123) returns False rather than raising (CPython behavior, NOT an error path)"""
import os.path

# A non-string/path-like argument does not raise; exists() swallows the
# error and reports False (CPython 3.12 behavior).
assert os.path.exists(123) == False, "exists(int) returns False, no raise"

print("exists_nonstring_returns_false_no_raise OK")
