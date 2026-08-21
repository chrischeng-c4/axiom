# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "atof_non_numeric_raises"
# subject = "locale.atof"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.atof: atof_non_numeric_raises (errors)."""
import locale

_raised = False
try:
    locale.atof("not a number")
except ValueError:
    _raised = True
assert _raised, "atof_non_numeric_raises: expected ValueError"
print("atof_non_numeric_raises OK")
