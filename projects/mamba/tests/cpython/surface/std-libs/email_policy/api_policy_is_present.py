# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_policy"
# dimension = "surface"
# case = "api_policy_is_present"
# subject = "email.policy.Policy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.policy.Policy: api_policy_is_present (surface)."""
import email.policy

assert hasattr(email.policy, "Policy")
print("api_policy_is_present OK")
