# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "api_summarize_address_range_is_present"
# subject = "ipaddress.summarize_address_range"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ipaddress.summarize_address_range: api_summarize_address_range_is_present (surface)."""
import ipaddress

assert hasattr(ipaddress, "summarize_address_range")
print("api_summarize_address_range_is_present OK")
