# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_policy"
# dimension = "surface"
# case = "api_default_is_present"
# subject = "email.policy.default"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.policy.default: api_default_is_present (surface)."""
import email.policy

assert hasattr(email.policy, "default")
print("api_default_is_present OK")
