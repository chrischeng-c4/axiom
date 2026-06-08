# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_alert_description_handshake_failure_is_present"
# subject = "ssl.ALERT_DESCRIPTION_HANDSHAKE_FAILURE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.ALERT_DESCRIPTION_HANDSHAKE_FAILURE: api_alert_description_handshake_failure_is_present (surface)."""
import ssl

assert hasattr(ssl, "ALERT_DESCRIPTION_HANDSHAKE_FAILURE")
print("api_alert_description_handshake_failure_is_present OK")
