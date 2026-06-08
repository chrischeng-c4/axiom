# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "api_utils_is_present"
# subject = "email.utils"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.utils: api_utils_is_present (surface)."""
import email.utils

assert hasattr(email, "utils")
print("api_utils_is_present OK")
