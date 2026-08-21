# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "getlocale_is_callable"
# subject = "locale.getlocale"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""locale.getlocale: getlocale_is_callable (surface)."""
import locale

assert callable(locale.getlocale)
print("getlocale_is_callable OK")
