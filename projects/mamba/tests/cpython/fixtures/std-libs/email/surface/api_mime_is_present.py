# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "api_mime_is_present"
# subject = "email.mime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.mime: api_mime_is_present (surface)."""
import email.mime

assert hasattr(email, "mime")
print("api_mime_is_present OK")
