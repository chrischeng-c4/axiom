# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_policy"
# dimension = "surface"
# case = "api_compat32_is_present"
# subject = "email.policy.Compat32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.policy.Compat32: api_compat32_is_present (surface)."""
import email.policy

assert hasattr(email.policy, "Compat32")
print("api_compat32_is_present OK")
