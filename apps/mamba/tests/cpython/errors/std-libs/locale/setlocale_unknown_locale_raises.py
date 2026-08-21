# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "setlocale_unknown_locale_raises"
# subject = "locale.setlocale"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.setlocale: setlocale_unknown_locale_raises (errors)."""
import locale

_raised = False
try:
    locale.setlocale(locale.LC_ALL, "no_such_locale_xyzzy")
except locale.Error:
    _raised = True
assert _raised, "setlocale_unknown_locale_raises: expected locale.Error"
print("setlocale_unknown_locale_raises OK")
