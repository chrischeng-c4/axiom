# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "surface"
# case = "api_currency_is_present"
# subject = "locale.currency"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""locale.currency: api_currency_is_present (surface)."""
import locale

assert hasattr(locale, "currency")
print("api_currency_is_present OK")
