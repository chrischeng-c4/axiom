# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "policy_default_attr"
# subject = "email.policy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.policy: policy_default_attr (surface)."""
import email.policy

assert hasattr(email.policy, "default")
print("policy_default_attr OK")
