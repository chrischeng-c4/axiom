# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_utils"
# dimension = "surface"
# case = "api_formatdate_is_present"
# subject = "email.utils.formatdate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.utils.formatdate: api_formatdate_is_present (surface)."""
import email.utils

assert hasattr(email.utils, "formatdate")
print("api_formatdate_is_present OK")
