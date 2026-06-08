# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_utils"
# dimension = "surface"
# case = "api_parsedate_to_datetime_is_present"
# subject = "email.utils.parsedate_to_datetime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.utils.parsedate_to_datetime: api_parsedate_to_datetime_is_present (surface)."""
import email.utils

assert hasattr(email.utils, "parsedate_to_datetime")
print("api_parsedate_to_datetime_is_present OK")
