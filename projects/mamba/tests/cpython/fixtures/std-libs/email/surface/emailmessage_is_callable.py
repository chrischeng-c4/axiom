# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "emailmessage_is_callable"
# subject = "email.message.EmailMessage"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.message.EmailMessage: emailmessage_is_callable (surface)."""
import email.message

assert callable(email.message.EmailMessage)
print("emailmessage_is_callable OK")
