# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "security"
# case = "crlf_injection_in_method_rejected"
# subject = "http.client.HTTPConnection"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
"""http.client.HTTPConnection: an attacker-supplied HTTP method containing control characters is rejected by putrequest with ValueError, preventing request-line smuggling"""
import http.client as hc

# No network: the method token is validated before any socket I/O.
conn = hc.HTTPConnection("example.com")
attack_method = "GET\r\nHost: evil.example"
rejected = False
try:
    conn.putrequest(attack_method, "/")
except ValueError as e:
    rejected = True
    assert "control characters" in str(e), f"unexpected message: {str(e)!r}"
assert rejected, "control characters in method must be rejected with ValueError"

print("crlf_injection_in_method_rejected OK")
