# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "errors"
# case = "too_many_positional_subpatterns_typeerror"
# subject = "match.class_pattern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: too_many_positional_subpatterns_typeerror (errors)."""
pass

_raised = False
try:
    exec("class C:\n __match_args__ = ()\nmatch C():\n case C(_a): pass")
except TypeError:
    _raised = True
assert _raised, "too_many_positional_subpatterns_typeerror: expected TypeError"
print("too_many_positional_subpatterns_typeerror OK")
