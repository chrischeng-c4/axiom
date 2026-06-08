# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "policy_compat32_attr"
# subject = "email.policy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.policy: policy_compat32_attr (surface)."""
import email.policy

assert hasattr(email.policy, "compat32")
print("policy_compat32_attr OK")
