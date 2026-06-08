# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "security"
# case = "is_line_junk_redos_resistant"
# subject = "difflib.IS_LINE_JUNK"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.IS_LINE_JUNK: IS_LINE_JUNK must not catastrophically backtrack: a million leading tabs followed by '##' classifies quickly as non-junk"""
import difflib

_evil = "\t" * 1_000_000 + "##"
assert not difflib.IS_LINE_JUNK(_evil), "redos guard: must classify quickly as non-junk"
print("is_line_junk_redos_resistant OK")
