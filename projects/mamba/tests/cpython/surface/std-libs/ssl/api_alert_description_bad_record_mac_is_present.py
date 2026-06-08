# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_alert_description_bad_record_mac_is_present"
# subject = "ssl.ALERT_DESCRIPTION_BAD_RECORD_MAC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.ALERT_DESCRIPTION_BAD_RECORD_MAC: api_alert_description_bad_record_mac_is_present (surface)."""
import ssl

assert hasattr(ssl, "ALERT_DESCRIPTION_BAD_RECORD_MAC")
print("api_alert_description_bad_record_mac_is_present OK")
