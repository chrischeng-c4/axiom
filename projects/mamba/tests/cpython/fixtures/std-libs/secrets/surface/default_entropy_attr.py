# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "default_entropy_attr"
# subject = "secrets"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets: default_entropy_attr (surface)."""
import secrets

assert hasattr(secrets, "DEFAULT_ENTROPY")
print("default_entropy_attr OK")
