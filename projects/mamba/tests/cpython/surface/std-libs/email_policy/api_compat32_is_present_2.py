# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_policy"
# dimension = "surface"
# case = "api_compat32_is_present_2"
# subject = "email.policy.compat32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.policy.compat32: api_compat32_is_present_2 (surface)."""
import email.policy

assert hasattr(email.policy, "compat32")
print("api_compat32_is_present_2 OK")
