# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "api_message_from_bytes_is_present"
# subject = "email.message_from_bytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.message_from_bytes: api_message_from_bytes_is_present (surface)."""
import email

assert hasattr(email, "message_from_bytes")
print("api_message_from_bytes_is_present OK")
