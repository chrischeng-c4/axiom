# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_alert_description_bad_certificate_status_response_is_present"
# subject = "ssl.ALERT_DESCRIPTION_BAD_CERTIFICATE_STATUS_RESPONSE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.ALERT_DESCRIPTION_BAD_CERTIFICATE_STATUS_RESPONSE: api_alert_description_bad_certificate_status_response_is_present (surface)."""
import ssl

assert hasattr(ssl, "ALERT_DESCRIPTION_BAD_CERTIFICATE_STATUS_RESPONSE")
print("api_alert_description_bad_certificate_status_response_is_present OK")
