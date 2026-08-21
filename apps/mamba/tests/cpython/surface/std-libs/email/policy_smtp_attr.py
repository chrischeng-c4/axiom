# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "policy_smtp_attr"
# subject = "email.policy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.policy: policy_smtp_attr (surface)."""
import email.policy

assert hasattr(email.policy, "SMTP")
print("policy_smtp_attr OK")
