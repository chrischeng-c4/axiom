# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_alert_description_decode_error_is_present"
# subject = "ssl.ALERT_DESCRIPTION_DECODE_ERROR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.ALERT_DESCRIPTION_DECODE_ERROR: api_alert_description_decode_error_is_present (surface)."""
import ssl

assert hasattr(ssl, "ALERT_DESCRIPTION_DECODE_ERROR")
print("api_alert_description_decode_error_is_present OK")
