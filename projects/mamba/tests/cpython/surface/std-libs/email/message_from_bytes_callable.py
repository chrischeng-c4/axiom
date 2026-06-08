# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "message_from_bytes_callable"
# subject = "email.message_from_bytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.message_from_bytes: message_from_bytes_callable (surface)."""
import email

assert callable(email.message_from_bytes)
print("message_from_bytes_callable OK")
