# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "status_constants_equal_int_values"
# subject = "http.client.OK"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
"""http.client.OK: the named status constants (OK, CREATED, NO_CONTENT, BAD_REQUEST, UNAUTHORIZED, FORBIDDEN, NOT_FOUND, INTERNAL_SERVER_ERROR) compare equal to their canonical integer codes"""
import http.client as hc

codes = [
    (hc.OK, 200),
    (hc.CREATED, 201),
    (hc.NO_CONTENT, 204),
    (hc.BAD_REQUEST, 400),
    (hc.UNAUTHORIZED, 401),
    (hc.FORBIDDEN, 403),
    (hc.NOT_FOUND, 404),
    (hc.INTERNAL_SERVER_ERROR, 500),
]
for const, value in codes:
    assert const == value, f"status code {value}: {const!r} != {value}"

print("status_constants_equal_int_values OK")
