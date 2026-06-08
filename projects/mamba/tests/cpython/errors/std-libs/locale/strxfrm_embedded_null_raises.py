# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "errors"
# case = "strxfrm_embedded_null_raises"
# subject = "locale.strxfrm"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale.strxfrm: strxfrm_embedded_null_raises (errors)."""
import locale

_raised = False
try:
    locale.strxfrm("a\x00")
except ValueError:
    _raised = True
assert _raised, "strxfrm_embedded_null_raises: expected ValueError"
print("strxfrm_embedded_null_raises OK")
