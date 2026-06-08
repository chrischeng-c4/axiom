# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "compare_digest_is_callable"
# subject = "secrets.compare_digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.compare_digest: compare_digest_is_callable (surface)."""
import secrets

assert callable(secrets.compare_digest)
print("compare_digest_is_callable OK")
