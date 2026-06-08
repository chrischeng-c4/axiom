# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_utils"
# dimension = "surface"
# case = "api_make_msgid_is_present"
# subject = "email.utils.make_msgid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.utils.make_msgid: api_make_msgid_is_present (surface)."""
import email.utils

assert hasattr(email.utils, "make_msgid")
print("api_make_msgid_is_present OK")
