# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "attrgetter_missing_attr_attributeerror"
# subject = "operator.attrgetter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.attrgetter: attrgetter_missing_attr_attributeerror (errors)."""
import operator

_raised = False
try:
    operator.attrgetter("foo")(object())
except AttributeError:
    _raised = True
assert _raised, "attrgetter_missing_attr_attributeerror: expected AttributeError"
print("attrgetter_missing_attr_attributeerror OK")
