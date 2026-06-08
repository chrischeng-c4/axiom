# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "behavior"
# case = "httpstatus_phrase_text"
# subject = "http.HTTPStatus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
"""http.HTTPStatus: each HTTPStatus member exposes its canonical reason phrase: HTTPStatus.OK.phrase=='OK' and HTTPStatus.NOT_FOUND.phrase=='Not Found'"""
from http import HTTPStatus

assert HTTPStatus.OK.phrase == "OK", HTTPStatus.OK.phrase
assert HTTPStatus.NOT_FOUND.phrase == "Not Found", HTTPStatus.NOT_FOUND.phrase
assert HTTPStatus.NO_CONTENT.phrase == "No Content", HTTPStatus.NO_CONTENT.phrase
assert (
    HTTPStatus.INTERNAL_SERVER_ERROR.phrase == "Internal Server Error"
), HTTPStatus.INTERNAL_SERVER_ERROR.phrase

print("httpstatus_phrase_text OK")
