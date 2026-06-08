# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "exception_hierarchy_rooted_at_httpexception"
# subject = "http.client.HTTPException"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
"""http.client.HTTPException: HTTPException subclasses Exception, and NotConnected / InvalidURL / UnknownProtocol / BadStatusLine / IncompleteRead all subclass HTTPException"""
import http.client as hc

assert issubclass(hc.HTTPException, Exception), "HTTPException < Exception"
assert issubclass(hc.NotConnected, hc.HTTPException), "NotConnected < HTTPException"
assert issubclass(hc.InvalidURL, hc.HTTPException), "InvalidURL < HTTPException"
assert issubclass(hc.UnknownProtocol, hc.HTTPException), "UnknownProtocol < HTTPException"
assert issubclass(hc.BadStatusLine, hc.HTTPException), "BadStatusLine < HTTPException"
assert issubclass(hc.IncompleteRead, hc.HTTPException), "IncompleteRead < HTTPException"

print("exception_hierarchy_rooted_at_httpexception OK")
