# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "request_get_post_method_selection"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.request.Request: a Request with no data defaults to GET; supplying a data body (even empty) makes get_method() return POST"""
from urllib.request import Request

assert Request("http://www.python.org").get_method() == "GET", "default GET"
assert Request("http://www.python.org", {}).get_method() == "POST", "data -> POST"

print("request_get_post_method_selection OK")
