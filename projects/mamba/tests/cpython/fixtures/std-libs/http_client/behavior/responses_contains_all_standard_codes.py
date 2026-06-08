# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "responses_contains_all_standard_codes"
# subject = "http.client.responses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.responses: the responses table contains every common standard status code (200, 201, 204, 206, 301, 302, 304, 400, 401, 403, 404, 405, 500, 503)"""
import http.client as hc

assert isinstance(hc.responses, dict), f"responses type = {type(hc.responses)!r}"
expected_present = [200, 201, 204, 206, 301, 302, 304, 400, 401, 403, 404, 405, 500, 503]
for code in expected_present:
    assert code in hc.responses, f"{code} in responses"

print("responses_contains_all_standard_codes OK")
