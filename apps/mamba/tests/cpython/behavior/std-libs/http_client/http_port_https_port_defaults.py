# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "http_port_https_port_defaults"
# subject = "http.client.HTTP_PORT"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.HTTP_PORT: the default port constants are HTTP_PORT == 80 and HTTPS_PORT == 443"""
import http.client as hc

assert hc.HTTP_PORT == 80, f"HTTP_PORT = {hc.HTTP_PORT!r}"
assert hc.HTTPS_PORT == 443, f"HTTPS_PORT = {hc.HTTPS_PORT!r}"

print("http_port_https_port_defaults OK")
