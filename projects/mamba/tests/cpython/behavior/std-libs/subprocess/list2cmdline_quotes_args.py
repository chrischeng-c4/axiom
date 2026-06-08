# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "list2cmdline_quotes_args"
# subject = "subprocess.list2cmdline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.list2cmdline: list2cmdline quotes arguments containing spaces and renders an empty argument as a quoted empty string"""
import subprocess

assert subprocess.list2cmdline(["a b c", "d", "e"]) == '"a b c" d e', "quote spaces"
assert subprocess.list2cmdline(["ab", ""]) == 'ab ""', "quote empty arg"
print("list2cmdline_quotes_args OK")
