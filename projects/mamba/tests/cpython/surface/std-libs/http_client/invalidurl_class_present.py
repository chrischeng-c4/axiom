# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "invalidurl_class_present"
# subject = "http.client.InvalidURL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.InvalidURL: invalidurl_class_present (surface)."""
import http.client

assert callable(http.client.InvalidURL)
print("invalidurl_class_present OK")
