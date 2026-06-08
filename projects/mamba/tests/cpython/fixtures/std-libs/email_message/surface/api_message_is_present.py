# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_message"
# dimension = "surface"
# case = "api_message_is_present"
# subject = "email.message.Message"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.message.Message: api_message_is_present (surface)."""
import email.message

assert hasattr(email.message, "Message")
print("api_message_is_present OK")
