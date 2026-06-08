# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "errors"
# case = "match_args_non_str_entry_typeerror"
# subject = "match.class_pattern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: match_args_non_str_entry_typeerror (errors)."""
pass

_raised = False
try:
    exec("class C:\n __match_args__ = (None,)\nmatch C():\n case C(_a): pass")
except TypeError:
    _raised = True
assert _raised, "match_args_non_str_entry_typeerror: expected TypeError"
print("match_args_non_str_entry_typeerror OK")
