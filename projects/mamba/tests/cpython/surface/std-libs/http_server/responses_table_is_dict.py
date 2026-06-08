# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "surface"
# case = "responses_table_is_dict"
# subject = "http.server.BaseHTTPRequestHandler.responses"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler.responses: responses_table_is_dict (surface)."""
import http.server

assert type(http.server.BaseHTTPRequestHandler.responses).__name__ == "dict"
print("responses_table_is_dict OK")
