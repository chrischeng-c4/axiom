# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "httpresponse_class_present"
# subject = "http.client.HTTPResponse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.HTTPResponse: httpresponse_class_present (surface)."""
import http.client

assert callable(http.client.HTTPResponse)
print("httpresponse_class_present OK")
