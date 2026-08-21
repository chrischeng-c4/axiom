# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "message_from_string_callable"
# subject = "email.message_from_string"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.message_from_string: message_from_string_callable (surface)."""
import email

assert callable(email.message_from_string)
print("message_from_string_callable OK")
