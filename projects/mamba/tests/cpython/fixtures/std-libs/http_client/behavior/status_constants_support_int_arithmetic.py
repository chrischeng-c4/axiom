# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "status_constants_support_int_arithmetic"
# subject = "http.client.OK"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.OK: status constants behave as ints under arithmetic: NOT_FOUND - OK == 204 and INTERNAL_SERVER_ERROR - BAD_REQUEST == 100"""
import http.client as hc

assert hc.NOT_FOUND - hc.OK == 204, f"404 - 200 = {hc.NOT_FOUND - hc.OK!r}"
assert hc.INTERNAL_SERVER_ERROR - hc.BAD_REQUEST == 100, "500 - 400 = 100"

print("status_constants_support_int_arithmetic OK")
