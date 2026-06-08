# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "mimetext_is_callable"
# subject = "email.mime.text.MIMEText"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.mime.text.MIMEText: mimetext_is_callable (surface)."""
import email.mime.text

assert callable(email.mime.text.MIMEText)
print("mimetext_is_callable OK")
