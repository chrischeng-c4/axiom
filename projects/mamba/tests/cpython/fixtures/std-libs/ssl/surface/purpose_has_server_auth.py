# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "purpose_has_server_auth"
# subject = "ssl.Purpose"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ssl.Purpose: purpose_has_server_auth (surface)."""
import ssl

assert hasattr(ssl.Purpose, "SERVER_AUTH")
print("purpose_has_server_auth OK")
