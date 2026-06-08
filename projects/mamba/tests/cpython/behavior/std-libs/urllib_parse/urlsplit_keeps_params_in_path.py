# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlsplit_keeps_params_in_path"
# subject = "urllib.parse.urlsplit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: urlsplit (unlike urlparse) does not break ';params' out of the path: urlsplit('https://example.com/a;params?q=1#frag').path is '/a;params'"""
from urllib.parse import urlsplit

s = urlsplit("https://example.com/a;params?q=1#frag")
assert s.scheme == "https", f"split scheme = {s.scheme!r}"
assert s.path == "/a;params", f"split path keeps params = {s.path!r}"
assert s.query == "q=1", f"split query = {s.query!r}"
assert s.fragment == "frag", f"split fragment = {s.fragment!r}"

print("urlsplit_keeps_params_in_path OK")
