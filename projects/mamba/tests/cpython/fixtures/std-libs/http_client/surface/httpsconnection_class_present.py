# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "httpsconnection_class_present"
# subject = "http.client.HTTPSConnection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.HTTPSConnection: httpsconnection_class_present (surface)."""
import http.client

assert callable(http.client.HTTPSConnection)
print("httpsconnection_class_present OK")
