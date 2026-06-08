# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_verify_default_is_present"
# subject = "ssl.VERIFY_DEFAULT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.VERIFY_DEFAULT: api_verify_default_is_present (surface)."""
import ssl

assert hasattr(ssl, "VERIFY_DEFAULT")
print("api_verify_default_is_present OK")
