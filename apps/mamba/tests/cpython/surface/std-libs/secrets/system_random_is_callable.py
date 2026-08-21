# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "system_random_is_callable"
# subject = "secrets.SystemRandom"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.SystemRandom: system_random_is_callable (surface)."""
import secrets

assert callable(secrets.SystemRandom)
print("system_random_is_callable OK")
