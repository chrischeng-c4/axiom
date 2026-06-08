# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "api_compare_digest_is_present"
# subject = "secrets.compare_digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""secrets.compare_digest: api_compare_digest_is_present (surface)."""
import secrets

assert hasattr(secrets, "compare_digest")
print("api_compare_digest_is_present OK")
