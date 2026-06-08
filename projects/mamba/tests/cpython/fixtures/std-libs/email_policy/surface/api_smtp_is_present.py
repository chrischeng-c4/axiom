# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_policy"
# dimension = "surface"
# case = "api_smtp_is_present"
# subject = "email.policy.SMTP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.policy.SMTP: api_smtp_is_present (surface)."""
import email.policy

assert hasattr(email.policy, "SMTP")
print("api_smtp_is_present OK")
