# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "netloc_absent_derived_attrs_none"
# subject = "urllib.parse.urlsplit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: with no '//' authority the netloc is '' and every derived attribute (username/hostname/port) is None: urlsplit('sip:alice@atlanta.com;maddr=239.255.255.1;ttl=15')"""
from urllib.parse import urlsplit

p = urlsplit("sip:alice@atlanta.com;maddr=239.255.255.1;ttl=15")
assert p.netloc == "", f"netloc = {p.netloc!r}"
assert p.username is None, f"username = {p.username!r}"
assert p.hostname is None, f"hostname = {p.hostname!r}"
assert p.port is None, f"port = {p.port!r}"

print("netloc_absent_derived_attrs_none OK")
