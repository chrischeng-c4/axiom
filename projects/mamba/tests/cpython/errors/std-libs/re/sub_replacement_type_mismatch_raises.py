# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "errors"
# case = "sub_replacement_type_mismatch_raises"
# subject = "re.Pattern.sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Pattern.sub: a str-compiled pattern's sub() rejects a bytes replacement against a str subject with TypeError"""
import re

str_pat = re.compile(".")

# A bytes replacement against a str subject is a type mismatch.
try:
    str_pat.sub(b"b", "c")
    raise AssertionError("bytes replacement on str subject should raise TypeError")
except TypeError:
    pass

print("sub_replacement_type_mismatch_raises OK")
