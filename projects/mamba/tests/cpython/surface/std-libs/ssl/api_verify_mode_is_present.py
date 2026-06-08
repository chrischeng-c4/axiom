# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_verify_mode_is_present"
# subject = "ssl.VerifyMode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.VerifyMode: api_verify_mode_is_present (surface)."""
import ssl

assert hasattr(ssl, "VerifyMode")
print("api_verify_mode_is_present OK")
