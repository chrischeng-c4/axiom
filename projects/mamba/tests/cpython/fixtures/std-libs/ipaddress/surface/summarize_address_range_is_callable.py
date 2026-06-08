# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "summarize_address_range_is_callable"
# subject = "ipaddress.summarize_address_range"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ipaddress.summarize_address_range: summarize_address_range_is_callable (surface)."""
import ipaddress

assert callable(ipaddress.summarize_address_range)
print("summarize_address_range_is_callable OK")
