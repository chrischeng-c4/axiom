# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_verify_flags_is_present"
# subject = "ssl.VerifyFlags"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.VerifyFlags: api_verify_flags_is_present (surface)."""
import ssl

assert hasattr(ssl, "VerifyFlags")
print("api_verify_flags_is_present OK")
