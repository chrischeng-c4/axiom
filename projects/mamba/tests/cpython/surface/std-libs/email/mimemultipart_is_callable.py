# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "mimemultipart_is_callable"
# subject = "email.mime.multipart.MIMEMultipart"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.mime.multipart.MIMEMultipart: mimemultipart_is_callable (surface)."""
import email.mime.multipart

assert callable(email.mime.multipart.MIMEMultipart)
print("mimemultipart_is_callable OK")
