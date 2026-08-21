# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "request_host_path_port"
# subject = "http.cookiejar.request_host"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.request_host: request_host/request_path/request_port extract host (URL host wins over Host: header, port stripped), path (params kept, query/fragment dropped, '/' default), and port (explicit URL port wins, else DEFAULT_HTTP_PORT '80')"""
import urllib.request
from http.cookiejar import (
    request_host,
    request_path,
    request_port,
    DEFAULT_HTTP_PORT,
)

# The default HTTP port is the string "80".
assert DEFAULT_HTTP_PORT == "80", DEFAULT_HTTP_PORT

# request_host: the URL host wins over the Host: header; an IP-literal is verbatim.
req = urllib.request.Request("http://1.1.1.1/", headers={"Host": "www.acme.com:80"})
assert request_host(req) == "1.1.1.1", request_host(req)
req = urllib.request.Request("http://www.acme.com/", headers={"Host": "irrelevant.com"})
assert request_host(req) == "www.acme.com", request_host(req)

# A port in the URL host is stripped from request_host.
req = urllib.request.Request(
    "http://www.acme.com:2345/resource.html", headers={"Host": "www.acme.com:5432"}
)
assert request_host(req) == "www.acme.com", request_host(req)

# request_path: path plus params, dropping the query and fragment.
req = urllib.request.Request(
    "http://www.example.com/rheum/rhaponticum;foo=bar;sing=song?a=b&c=d#ni"
)
assert request_path(req) == "/rheum/rhaponticum;foo=bar;sing=song", request_path(req)
req = urllib.request.Request("http://www.example.com/rheum/rhaponticum?a=b&c=d#ni")
assert request_path(req) == "/rheum/rhaponticum", request_path(req)

# A URL with no path component yields "/".
req = urllib.request.Request("http://www.example.com")
assert request_path(req) == "/", request_path(req)

# request_port: an explicit port in the URL host wins over the Host: header.
req = urllib.request.Request(
    "http://www.acme.com:1234/", headers={"Host": "www.acme.com:4321"}
)
assert request_port(req) == "1234", request_port(req)

# With no explicit URL port, request_port falls back to DEFAULT_HTTP_PORT.
req = urllib.request.Request("http://www.acme.com/", headers={"Host": "www.acme.com:4321"})
assert request_port(req) == DEFAULT_HTTP_PORT, request_port(req)

print("request_host_path_port OK")
