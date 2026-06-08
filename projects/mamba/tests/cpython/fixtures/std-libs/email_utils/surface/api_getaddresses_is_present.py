# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_utils"
# dimension = "surface"
# case = "api_getaddresses_is_present"
# subject = "email.utils.getaddresses"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.utils.getaddresses: api_getaddresses_is_present (surface)."""
import email.utils

assert hasattr(email.utils, "getaddresses")
print("api_getaddresses_is_present OK")
