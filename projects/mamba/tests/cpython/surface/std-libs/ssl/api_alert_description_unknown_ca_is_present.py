# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_alert_description_unknown_ca_is_present"
# subject = "ssl.ALERT_DESCRIPTION_UNKNOWN_CA"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.ALERT_DESCRIPTION_UNKNOWN_CA: api_alert_description_unknown_ca_is_present (surface)."""
import ssl

assert hasattr(ssl, "ALERT_DESCRIPTION_UNKNOWN_CA")
print("api_alert_description_unknown_ca_is_present OK")
