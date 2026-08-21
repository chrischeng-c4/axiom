# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "strcoll_embedded_null_left_raises"
# subject = "locale.strcoll"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.strcoll: strcoll_embedded_null_left_raises (errors)."""
import locale

_raised = False
try:
    locale.strcoll("a\x00", "a")
except ValueError:
    _raised = True
assert _raised, "strcoll_embedded_null_left_raises: expected ValueError"
print("strcoll_embedded_null_left_raises OK")
