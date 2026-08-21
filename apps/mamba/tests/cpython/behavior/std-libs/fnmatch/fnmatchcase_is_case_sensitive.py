# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "fnmatchcase_is_case_sensitive"
# subject = "fnmatch.fnmatchcase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatchcase: fnmatchcase is strictly case-sensitive regardless of OS: 'README.MD' matches '*.MD', but 'readme.md' does not match '*.MD' (only '*.md')"""
import fnmatch

assert fnmatch.fnmatchcase("README.MD", "*.MD"), "upper suffix matches upper pattern"
assert not fnmatch.fnmatchcase("readme.md", "*.MD"), "case mismatch does not match"
assert fnmatch.fnmatchcase("readme.md", "*.md"), "lower suffix matches lower pattern"

print("fnmatchcase_is_case_sensitive OK")
