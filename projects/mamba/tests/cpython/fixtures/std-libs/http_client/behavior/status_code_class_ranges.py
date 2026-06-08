# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "status_code_class_ranges"
# subject = "http.client.OK"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.OK: status codes classify by hundreds range: 1xx informational, 2xx success, 3xx redirect, 4xx client error, 5xx server error via integer comparison"""
import http.client as hc

info_codes = [100, 101, 102]
success_codes = [hc.OK, hc.CREATED, hc.NO_CONTENT, 206]
redirect_codes = [301, 302, 303, 304, 307, 308]
client_error_codes = [400, 401, 403, 404, 409, 422]
server_error_codes = [500, 501, 502, 503]

for c in info_codes:
    assert 100 <= c < 200, f"info: {c}"
for c in success_codes:
    assert 200 <= c < 300, f"success: {c}"
for c in redirect_codes:
    assert 300 <= c < 400, f"redirect: {c}"
for c in client_error_codes:
    assert 400 <= c < 500, f"client error: {c}"
for c in server_error_codes:
    assert 500 <= c < 600, f"server error: {c}"

print("status_code_class_ranges OK")
