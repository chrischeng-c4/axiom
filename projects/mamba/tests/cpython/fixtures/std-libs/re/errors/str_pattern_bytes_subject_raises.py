# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "str_pattern_bytes_subject_raises"
# subject = "re.Pattern.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Pattern.match: a str-compiled pattern rejects a bytes subject with TypeError, and a bytes-compiled pattern rejects a str subject with TypeError"""
import re

str_pat = re.compile(".")
bytes_pat = re.compile(b".")

# A str pattern cannot scan a bytes subject.
try:
    str_pat.match(b"b")
    raise AssertionError("str pattern on bytes subject should raise TypeError")
except TypeError:
    pass

# A bytes pattern cannot scan a str subject.
try:
    bytes_pat.match("b")
    raise AssertionError("bytes pattern on str subject should raise TypeError")
except TypeError:
    pass

print("str_pattern_bytes_subject_raises OK")
