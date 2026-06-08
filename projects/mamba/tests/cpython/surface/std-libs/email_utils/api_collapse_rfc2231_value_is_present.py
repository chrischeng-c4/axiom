# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_utils"
# dimension = "surface"
# case = "api_collapse_rfc2231_value_is_present"
# subject = "email.utils.collapse_rfc2231_value"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.utils.collapse_rfc2231_value: api_collapse_rfc2231_value_is_present (surface)."""
import email.utils

assert hasattr(email.utils, "collapse_rfc2231_value")
print("api_collapse_rfc2231_value_is_present OK")
