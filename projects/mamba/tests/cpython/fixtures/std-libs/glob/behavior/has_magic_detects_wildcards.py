# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "has_magic_detects_wildcards"
# subject = "glob.has_magic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.has_magic: has_magic is True iff the pattern contains a glob metachar (* ? [): True for 'a*', 'a?b', 'x[1]'; False for 'plain' and 'path/to/file'"""
import glob

for pattern, expected in [
    ("plain", False),
    ("path/to/file", False),
    ("a*", True),
    ("a?b", True),
    ("x[1]", True),
]:
    assert glob.has_magic(pattern) == expected, (pattern, glob.has_magic(pattern), expected)

print("has_magic_detects_wildcards OK")
