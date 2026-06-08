# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_alert_description_protocol_version_is_present"
# subject = "ssl.ALERT_DESCRIPTION_PROTOCOL_VERSION"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.ALERT_DESCRIPTION_PROTOCOL_VERSION: api_alert_description_protocol_version_is_present (surface)."""
import ssl

assert hasattr(ssl, "ALERT_DESCRIPTION_PROTOCOL_VERSION")
print("api_alert_description_protocol_version_is_present OK")
