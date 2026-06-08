# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_policy"
# dimension = "surface"
# case = "api_strict_is_present"
# subject = "email.policy.strict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.policy.strict: api_strict_is_present (surface)."""
import email.policy

assert hasattr(email.policy, "strict")
print("api_strict_is_present OK")
