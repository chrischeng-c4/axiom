# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_utils"
# dimension = "surface"
# case = "api_unquote_is_present"
# subject = "email.utils.unquote"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.utils.unquote: api_unquote_is_present (surface)."""
import email.utils

assert hasattr(email.utils, "unquote")
print("api_unquote_is_present OK")
