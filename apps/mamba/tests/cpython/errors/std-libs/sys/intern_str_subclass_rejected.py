# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "errors"
# case = "intern_str_subclass_rejected"
# subject = "sys.intern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.intern: sys.intern only accepts an exact str; a str subclass instance raises TypeError"""
import sys


class _SubStr(str):
    def __hash__(self):
        return 123


_raised = False
try:
    sys.intern(_SubStr("abc"))
except TypeError:
    _raised = True
assert _raised, "intern_str_subclass_rejected: expected TypeError"
print("intern_str_subclass_rejected OK")
