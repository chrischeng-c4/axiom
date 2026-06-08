# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "api_charset_is_present"
# subject = "email.charset"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.charset: api_charset_is_present (surface)."""
import email.charset

assert hasattr(email, "charset")
print("api_charset_is_present OK")
