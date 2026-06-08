# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_alert_description_is_present"
# subject = "ssl.AlertDescription"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.AlertDescription: api_alert_description_is_present (surface)."""
import ssl

assert hasattr(ssl, "AlertDescription")
print("api_alert_description_is_present OK")
