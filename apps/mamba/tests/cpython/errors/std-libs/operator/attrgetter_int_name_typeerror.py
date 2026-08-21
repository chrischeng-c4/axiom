# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "attrgetter_int_name_typeerror"
# subject = "operator.attrgetter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.attrgetter: attrgetter_int_name_typeerror (errors)."""
import operator

_raised = False
try:
    operator.attrgetter(2)
except TypeError:
    _raised = True
assert _raised, "attrgetter_int_name_typeerror: expected TypeError"
print("attrgetter_int_name_typeerror OK")
