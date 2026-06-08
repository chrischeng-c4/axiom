# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "scheme_and_host_lowercased_userinfo_preserved"
# subject = "urllib.parse.urlsplit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: urlsplit lowercases scheme and hostname but preserves netloc/userinfo case; a leading-zero port ':080' normalizes to 80 and the last '@' splits userinfo from host"""
from urllib.parse import urlsplit

p = urlsplit("HTTP://User:Pass@WWW.PYTHON.ORG:080/doc/?q=yes#frag")
assert p.scheme == "http", f"scheme lowercased = {p.scheme!r}"
assert p.hostname == "www.python.org", f"hostname lowercased = {p.hostname!r}"
assert p.netloc == "User:Pass@WWW.PYTHON.ORG:080", f"netloc preserves case = {p.netloc!r}"
assert p.username == "User", f"username = {p.username!r}"
assert p.password == "Pass", f"password = {p.password!r}"
assert p.port == 80, f"leading-zero port normalized = {p.port!r}"

u = urlsplit("http://User@example.com:Pass@www.python.org:443/")
assert u.username == "User@example.com", f"last @ splits host = {u.username!r}"
assert u.password == "Pass", f"password = {u.password!r}"
assert u.hostname == "www.python.org", f"hostname = {u.hostname!r}"
assert u.port == 443, f"port = {u.port!r}"

print("scheme_and_host_lowercased_userinfo_preserved OK")
