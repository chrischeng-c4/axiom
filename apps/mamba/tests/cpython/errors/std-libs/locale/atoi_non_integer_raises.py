# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "atoi_non_integer_raises"
# subject = "locale.atoi"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.atoi: atoi_non_integer_raises (errors)."""
import locale

_raised = False
try:
    locale.atoi("not an int")
except ValueError:
    _raised = True
assert _raised, "atoi_non_integer_raises: expected ValueError"
print("atoi_non_integer_raises OK")
