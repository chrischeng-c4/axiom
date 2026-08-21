# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "httpconnection_class_present"
# subject = "http.client.HTTPConnection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.HTTPConnection: httpconnection_class_present (surface)."""
import http.client

assert callable(http.client.HTTPConnection)
print("httpconnection_class_present OK")
