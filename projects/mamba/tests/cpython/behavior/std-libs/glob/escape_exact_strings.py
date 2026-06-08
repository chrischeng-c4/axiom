# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "escape_exact_strings"
# subject = "glob.escape"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.escape: escape wraps each glob metacharacter in a literal character class: escape('plain')=='plain', escape('a*')=='a[*]', escape('a?b')=='a[?]b', escape('file*.txt')=='file[*].txt'"""
import glob

for pattern, expected in [
    ("plain", "plain"),
    ("a*", "a[*]"),
    ("a?b", "a[?]b"),
    ("file*.txt", "file[*].txt"),
]:
    assert glob.escape(pattern) == expected, (pattern, glob.escape(pattern), expected)

print("escape_exact_strings OK")
