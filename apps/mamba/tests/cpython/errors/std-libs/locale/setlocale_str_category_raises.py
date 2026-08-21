# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "setlocale_str_category_raises"
# subject = "locale.setlocale"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.setlocale: setlocale_str_category_raises (errors)."""
import locale

_raised = False
try:
    locale.setlocale("not_a_category", "C")
except (TypeError, locale.Error):
    _raised = True
assert _raised, "setlocale_str_category_raises: expected (TypeError, locale.Error)"
print("setlocale_str_category_raises OK")
