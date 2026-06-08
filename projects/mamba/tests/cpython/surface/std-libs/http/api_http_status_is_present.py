# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http"
# dimension = "surface"
# case = "api_http_status_is_present"
# subject = "http.HTTPStatus"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.HTTPStatus: api_http_status_is_present (surface)."""
import http

assert hasattr(http, "HTTPStatus")
print("api_http_status_is_present OK")
