# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "choice_is_callable"
# subject = "secrets.choice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.choice: choice_is_callable (surface)."""
import secrets

assert callable(secrets.choice)
print("choice_is_callable OK")
