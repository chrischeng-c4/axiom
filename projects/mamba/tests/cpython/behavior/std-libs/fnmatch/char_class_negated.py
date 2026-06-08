# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "char_class_negated"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatchcase: [!a-z] is a negated class: a digit '0' matches, a lowercase letter 'a' does not"""
import fnmatch

assert fnmatch.fnmatchcase("file_0.txt", "file_[!a-z].txt"), "negated class matches digit"
assert not fnmatch.fnmatchcase("file_a.txt", "file_[!a-z].txt"), "negated class excludes letter"

print("char_class_negated OK")
