# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_policy"
# dimension = "surface"
# case = "api_email_policy_is_present"
# subject = "email.policy.EmailPolicy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.policy.EmailPolicy: api_email_policy_is_present (surface)."""
import email.policy

assert hasattr(email.policy, "EmailPolicy")
print("api_email_policy_is_present OK")
