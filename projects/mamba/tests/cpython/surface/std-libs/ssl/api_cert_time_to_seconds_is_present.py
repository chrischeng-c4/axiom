# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_cert_time_to_seconds_is_present"
# subject = "ssl.cert_time_to_seconds"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.cert_time_to_seconds: api_cert_time_to_seconds_is_present (surface)."""
import ssl

assert hasattr(ssl, "cert_time_to_seconds")
print("api_cert_time_to_seconds_is_present OK")
