# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "real_world"
# case = "http_status_codes"
# subject = "http.HTTPStatus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.HTTPStatus: the HTTPStatus enum exposes the canonical numeric codes consumed alongside urllib (OK=200, NOT_FOUND=404, INTERNAL_SERVER_ERROR=500, BAD_REQUEST=400, UNAUTHORIZED=401, FORBIDDEN=403, CREATED=201, NO_CONTENT=204)"""
from http import HTTPStatus

assert int(HTTPStatus.OK) == 200
assert int(HTTPStatus.CREATED) == 201
assert int(HTTPStatus.NO_CONTENT) == 204
assert int(HTTPStatus.BAD_REQUEST) == 400
assert int(HTTPStatus.UNAUTHORIZED) == 401
assert int(HTTPStatus.FORBIDDEN) == 403
assert int(HTTPStatus.NOT_FOUND) == 404
assert int(HTTPStatus.INTERNAL_SERVER_ERROR) == 500

print("http_status_codes OK")
