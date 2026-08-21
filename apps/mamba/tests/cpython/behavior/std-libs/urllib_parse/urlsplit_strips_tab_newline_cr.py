# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlsplit_strips_tab_newline_cr"
# subject = "urllib.parse.urlsplit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: WHATWG-aligned cleanup: urlsplit removes embedded tab/newline/CR from every component and trims leading C0 control bytes, while a trailing space inside the body round-trips through urlunsplit"""
from urllib.parse import urlsplit, urlunsplit

url = ("http\t://www.python\n.org\t/java\nscript:\talert('msg\r\n')"
       "/?query\n=\tsomething#frag\nment")
p = urlsplit(url)
assert p.scheme == "http", f"scheme = {p.scheme!r}"
assert p.netloc == "www.python.org", f"netloc = {p.netloc!r}"
assert p.path == "/javascript:alert('msg')/", f"path = {p.path!r}"
assert p.query == "query=something", f"query = {p.query!r}"
assert p.fragment == "fragment", f"fragment = {p.fragment!r}"

noise = bytes(range(0, 33)).decode("utf-8")
base = "http://User:Pass@www.python.org:080/doc/?query=yes#frag"
p3 = urlsplit(noise + base)
assert p3.scheme == "http", f"scheme after C0 trim = {p3.scheme!r}"
assert p3.port == 80, f"port = {p3.port!r}"

p4 = urlsplit("www.pypi.org ")
assert urlunsplit(p4) == "www.pypi.org ", f"trailing space round-trips = {urlunsplit(p4)!r}"

print("urlsplit_strips_tab_newline_cr OK")
