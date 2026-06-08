# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "class_shortcut_escapes"
# subject = "re.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.search: the class shortcuts \\d \\D \\w \\W \\s \\S match digit/non-digit, word/non-word, space/non-space respectively, for both str and bytes patterns"""
import re

assert re.search(r"\d\D\w\W\s\S", "1aa! a").group(0) == "1aa! a", "class shortcuts str"
assert re.search(rb"\d\D\w\W\s\S", b"1aa! a").group(0) == b"1aa! a", "class shortcuts bytes"

print("class_shortcut_escapes OK")
