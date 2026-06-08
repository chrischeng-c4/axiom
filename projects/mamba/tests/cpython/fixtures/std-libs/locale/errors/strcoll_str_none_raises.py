# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "strcoll_str_none_raises"
# subject = "locale.strcoll"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.strcoll: strcoll_str_none_raises (errors)."""
import locale

_raised = False
try:
    locale.strcoll("a", None)
except TypeError:
    _raised = True
assert _raised, "strcoll_str_none_raises: expected TypeError"
print("strcoll_str_none_raises OK")
