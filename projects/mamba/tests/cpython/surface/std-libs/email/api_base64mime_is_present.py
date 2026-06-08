# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "api_base64mime_is_present"
# subject = "email.base64mime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.base64mime: api_base64mime_is_present (surface)."""
import email.base64mime

assert hasattr(email, "base64mime")
print("api_base64mime_is_present OK")
