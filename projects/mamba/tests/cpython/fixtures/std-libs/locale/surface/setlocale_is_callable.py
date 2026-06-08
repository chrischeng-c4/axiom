# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "setlocale_is_callable"
# subject = "locale.setlocale"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""locale.setlocale: setlocale_is_callable (surface)."""
import locale

assert callable(locale.setlocale)
print("setlocale_is_callable OK")
