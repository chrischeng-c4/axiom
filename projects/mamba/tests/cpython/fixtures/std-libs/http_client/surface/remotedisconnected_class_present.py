# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "remotedisconnected_class_present"
# subject = "http.client.RemoteDisconnected"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.RemoteDisconnected: remotedisconnected_class_present (surface)."""
import http.client

assert callable(http.client.RemoteDisconnected)
print("remotedisconnected_class_present OK")
