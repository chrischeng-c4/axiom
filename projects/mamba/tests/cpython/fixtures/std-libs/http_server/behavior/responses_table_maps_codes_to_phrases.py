# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "responses_table_maps_codes_to_phrases"
# subject = "http.server.BaseHTTPRequestHandler.responses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.server.BaseHTTPRequestHandler.responses: BaseHTTPRequestHandler.responses[code] yields a (short, long) phrase pair whose short phrase matches the standard reason text for 200, 404, and 500"""
import http.server

responses = http.server.BaseHTTPRequestHandler.responses
for code, short in [(200, "OK"), (404, "Not Found"), (500, "Internal Server Error")]:
    entry = responses[code]
    assert isinstance(entry, tuple) and len(entry) == 2, (code, entry)
    assert entry[0] == short, (code, entry)
    assert isinstance(entry[1], str) and entry[1], (code, entry)

print("responses_table_maps_codes_to_phrases OK")
