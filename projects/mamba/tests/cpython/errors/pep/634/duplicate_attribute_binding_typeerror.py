# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "errors"
# case = "duplicate_attribute_binding_typeerror"
# subject = "match.class_pattern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: duplicate_attribute_binding_typeerror (errors)."""
pass

_raised = False
try:
    exec("class C:\n __match_args__ = ('a', 'a')\n a = None\nmatch C():\n case C(_p, _q): pass")
except TypeError:
    _raised = True
assert _raised, "duplicate_attribute_binding_typeerror: expected TypeError"
print("duplicate_attribute_binding_typeerror OK")
