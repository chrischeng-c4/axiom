# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "httpstatus_named_codes_equal_int_values"
# subject = "http.HTTPStatus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.HTTPStatus: the named HTTPStatus members equal their canonical integer codes: OK==200, NO_CONTENT==204, NOT_FOUND==404, INTERNAL_SERVER_ERROR==500"""
from http import HTTPStatus

for member, code in [
    (HTTPStatus.OK, 200),
    (HTTPStatus.NO_CONTENT, 204),
    (HTTPStatus.NOT_FOUND, 404),
    (HTTPStatus.INTERNAL_SERVER_ERROR, 500),
]:
    assert member == code, (member, code)
    assert member.value == code, (member, code)

print("httpstatus_named_codes_equal_int_values OK")
