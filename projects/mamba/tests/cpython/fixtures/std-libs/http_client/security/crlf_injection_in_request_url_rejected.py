# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "security"
# case = "crlf_injection_in_request_url_rejected"
# subject = "http.client.HTTPConnection"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
"""http.client.HTTPConnection: an attacker-supplied request URL containing CR/LF (header-injection / request-smuggling attempt) is rejected by putrequest with InvalidURL before any bytes are sent"""
import http.client as hc

# No network: putrequest validates the request line before any socket I/O.
conn = hc.HTTPConnection("example.com")
attack_url = "/foo\r\nHost: evil.example\r\nX-Injected: 1"
rejected = False
try:
    conn.putrequest("GET", attack_url)
except hc.InvalidURL as e:
    rejected = True
    assert "control characters" in str(e), f"unexpected message: {str(e)!r}"
assert rejected, "CRLF in request URL must be rejected with InvalidURL"

print("crlf_injection_in_request_url_rejected OK")
