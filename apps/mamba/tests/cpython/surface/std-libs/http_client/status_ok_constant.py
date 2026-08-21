# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "status_ok_constant"
# subject = "http.client.OK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.OK: status_ok_constant (surface)."""
import http.client

assert hasattr(http.client.OK, "real")
print("status_ok_constant OK")
