# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "message_message_attr"
# subject = "email.message"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.message: message_message_attr (surface)."""
import email.message

assert hasattr(email.message, "Message")
print("message_message_attr OK")
