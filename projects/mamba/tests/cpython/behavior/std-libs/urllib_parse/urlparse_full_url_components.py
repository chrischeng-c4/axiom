# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlparse_full_url_components"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: urlparse dissects a full authority URL into scheme/netloc/path/query/fragment and exposes derived username/password/hostname/port for 'https://user:pass@example.com:8080/path?key=val#frag'"""
from urllib.parse import urlparse

r = urlparse("https://user:pass@example.com:8080/path?key=val#frag")
assert r.scheme == "https", f"scheme = {r.scheme!r}"
assert r.netloc == "user:pass@example.com:8080", f"netloc = {r.netloc!r}"
assert r.path == "/path", f"path = {r.path!r}"
assert r.query == "key=val", f"query = {r.query!r}"
assert r.fragment == "frag", f"fragment = {r.fragment!r}"
assert r.username == "user", f"username = {r.username!r}"
assert r.password == "pass", f"password = {r.password!r}"
assert r.hostname == "example.com", f"hostname = {r.hostname!r}"
assert r.port == 8080, f"port = {r.port!r}"

print("urlparse_full_url_components OK")
