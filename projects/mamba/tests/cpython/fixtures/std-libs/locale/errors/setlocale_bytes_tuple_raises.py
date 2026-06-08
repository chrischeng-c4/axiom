# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "setlocale_bytes_tuple_raises"
# subject = "locale.setlocale"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.setlocale: setlocale_bytes_tuple_raises (errors)."""
import locale

_raised = False
try:
    locale.setlocale(locale.LC_ALL, (b"not", b"valid"))
except TypeError:
    _raised = True
assert _raised, "setlocale_bytes_tuple_raises: expected TypeError"
print("setlocale_bytes_tuple_raises OK")
