# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "responses_maps_codes_to_reason_phrases"
# subject = "http.client.responses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
"""http.client.responses: responses[code] yields the standard reason phrase for each canonical status code (200->'OK', 201->'Created', 404->'Not Found', 500->'Internal Server Error', ...)"""
import http.client as hc

phrases = {
    200: "OK",
    201: "Created",
    204: "No Content",
    301: "Moved Permanently",
    400: "Bad Request",
    401: "Unauthorized",
    403: "Forbidden",
    404: "Not Found",
    500: "Internal Server Error",
}
for code, phrase in phrases.items():
    assert phrase in hc.responses[code], \
        f"responses[{code}] has '{phrase}': {hc.responses[code]!r}"

print("responses_maps_codes_to_reason_phrases OK")
