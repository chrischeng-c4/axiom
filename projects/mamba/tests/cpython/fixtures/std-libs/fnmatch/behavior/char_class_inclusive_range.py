# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "char_class_inclusive_range"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatchcase: [a-z] matches an inclusive lowercase range: both endpoints 'a' and 'z' match, a digit '0' does not"""
import fnmatch

assert fnmatch.fnmatchcase("file_a.txt", "file_[a-z].txt"), "lowercase range start"
assert fnmatch.fnmatchcase("file_z.txt", "file_[a-z].txt"), "end of range"
assert not fnmatch.fnmatchcase("file_0.txt", "file_[a-z].txt"), "digit out of range"

print("char_class_inclusive_range OK")
