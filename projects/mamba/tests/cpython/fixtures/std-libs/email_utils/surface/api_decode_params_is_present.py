# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_utils"
# dimension = "surface"
# case = "api_decode_params_is_present"
# subject = "email.utils.decode_params"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.utils.decode_params: api_decode_params_is_present (surface)."""
import email.utils

assert hasattr(email.utils, "decode_params")
print("api_decode_params_is_present OK")
