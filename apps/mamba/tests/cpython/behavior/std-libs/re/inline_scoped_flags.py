# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "inline_scoped_flags"
# subject = "re.fullmatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.fullmatch: (?i) enables case-insensitivity globally while scoped (?i:...) limits it to that group and (?-x:...) re-enables whitespace significance inside"""
import re

# (?i) global inline ignorecase.
assert re.match(r"(?i)abc", "ABC").group(0) == "ABC", "(?i) inline ignorecase"
# Scoped (?i:...) limits the flag to that group.
assert re.fullmatch(r"a(?i:b)c", "aBc") is not None, "scoped (?i:) matches B"
assert re.fullmatch(r"a(?i:b)c", "AbC") is None, "scoped flag does not leak out"
# Scoped disable (?-x:...) re-enables whitespace significance inside.
assert re.fullmatch(r"(?x) a(?-x: b) c", "a bc") is not None, "scoped (?-x:) keeps space"

print("inline_scoped_flags OK")
